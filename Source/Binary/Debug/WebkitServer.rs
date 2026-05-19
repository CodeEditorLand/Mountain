//! Debug server for inspecting webview content via HTTP API.
//! Only compiled in debug builds.
//!
//! # Layered DebugServer (Mountain layer)
//!
//! This is the **Mountain** half of the dual-layer DebugServer. It exposes
//! an HTTP surface bound to `127.0.0.1` for live inspection and command
//! invocation against the running renderer (workbench webview).
//!
//! Activation is gated by the unified `DebugServer` env var:
//!
//! | Value                          | Mountain layer | Cocoon layer |
//! |--------------------------------|----------------|--------------|
//! | unset, `0`, `false`, `off`     | off            | off          |
//! | `1`, `true`, `on`              | on             | off (compat) |
//! | `mountain`, `m`                | on             | off          |
//! | `cocoon`, `c`, `eh`            | off            | on           |
//! | `both`, `all`                  | on             | on           |
//!
//! Ports: `DebugServerPort` (legacy alias `DebugServerPortMountain`,
//! default `9933`) — Mountain. Cocoon uses `DebugServerPortCocoon`
//! (default `9934`).
//!
//! ## Endpoints
//!
//! | Method | Path             | Purpose                                            |
//! |--------|------------------|----------------------------------------------------|
//! | GET    | `/health`        | Layer identity + capability advertisement          |
//! | GET    | `/layers`        | Discoverability: lists every reachable layer       |
//! | GET    | `/eval?js=…`     | Eval JS in the renderer; returns parsed JSON       |
//! | POST   | `/execute`       | Body `{js,target?}`; target=`renderer\|iframe:<id>` |
//! | GET    | `/iframes`       | Walk DOM iframes (src/id/name)                     |
//! | GET    | `/console`       | Drain the renderer console mirror buffer          |
//! | GET    | `/commands`      | Enumerate registered workbench commands           |
//! | POST   | `/command`       | Body `{id,args?}` — invoke a workbench command    |
//! | POST   | `/vscode/diff`   | Body `{left,right,title?}` — open diff editor      |
//! | GET    | `/extensions`    | Proxies Cocoon `/extensions` if reachable          |
//!
//! All responses are JSON. Mountain-layer endpoints execute in the renderer
//! via `WebviewWindow::eval_with_callback`. Cocoon-targeted requests are
//! HTTP-forwarded to the Cocoon DebugServer when present, transparently.

use std::{
	collections::HashMap,
	io::{self, BufRead, BufReader, Read, Write},
	net::TcpListener,
	sync::{Arc, Mutex},
	time::Duration,
};

use once_cell::sync::Lazy;
use serde_json::{Value, json};
use tauri::{WebviewWindow, Wry};
use url::Url;

/// Global storage for the webview window used by the debug server.
static WINDOW: Lazy<Mutex<Option<Arc<WebviewWindow<Wry>>>>> = Lazy::new(|| Mutex::new(None));

/// Parsed Mountain-layer activation mode. See module docs for the matrix.
#[derive(Copy, Clone, Debug)]
enum LayerMode {
	Off,
	Mountain,
	Cocoon,
	Both,
}

fn parse_mode() -> LayerMode {
	match std::env::var("DebugServer").ok().as_deref().map(str::trim) {
		None | Some("") | Some("0") | Some("false") | Some("off") | Some("no") => LayerMode::Off,
		Some(v) => {
			let v = v.to_ascii_lowercase();
			match v.as_str() {
				"mountain" | "m" | "native" | "rust" => LayerMode::Mountain,
				"cocoon" | "c" | "eh" | "extension-host" | "node" => LayerMode::Cocoon,
				"both" | "all" | "dual" => LayerMode::Both,
				// Legacy compat: 1/true/on enables Mountain-only.
				"1" | "true" | "on" | "yes" => LayerMode::Mountain,
				_ => LayerMode::Off,
			}
		}
	}
}

fn mountain_enabled(m: LayerMode) -> bool {
	matches!(m, LayerMode::Mountain | LayerMode::Both)
}

fn cocoon_enabled(m: LayerMode) -> bool {
	matches!(m, LayerMode::Cocoon | LayerMode::Both)
}

fn mountain_port() -> u16 {
	std::env::var("DebugServerPortMountain")
		.or_else(|_| std::env::var("DebugServerPort"))
		.ok()
		.and_then(|p| p.parse().ok())
		.unwrap_or(9933)
}

fn cocoon_port() -> u16 {
	std::env::var("DebugServerPortCocoon")
		.ok()
		.and_then(|p| p.parse().ok())
		.unwrap_or(9934)
}

/// Installs the Mountain-layer debug server and stores a reference to the
/// renderer webview window. Called once during app setup in debug builds.
///
/// Activation is gated by the unified `DebugServer` env var (see module
/// docs). When `cocoon`/`both` is selected, this function still installs
/// the window handle (so `/eval` keeps working for proxy requests) but
/// skips starting the Mountain HTTP listener.
pub fn install(window: &WebviewWindow<Wry>) {
	// Always store the window: even in cocoon-only mode the eval pipeline
	// stays useful for tests that imported this module directly.
	let mut guard = WINDOW.lock().unwrap();
	*guard = Some(Arc::new(window.clone()));
	drop(guard);

	let mode = parse_mode();
	if mountain_enabled(mode) {
		std::thread::spawn(|| start_server());
	}
	if cocoon_enabled(mode) {
		eprintln!(
			"[WebkitDebug] Cocoon layer requested (port {}). Cocoon must start its own listener.",
			cocoon_port()
		);
	}
}

/// Main server loop listening for TCP connections.
fn start_server() {
	let port = mountain_port();

	let listener = match TcpListener::bind(("127.0.0.1", port)) {
		Ok(l) => l,
		Err(e) => {
			eprintln!("[WebkitDebug] Failed to bind to 127.0.0.1:{}: {}", port, e);
			return;
		}
	};
	eprintln!(
		"[WebkitDebug] Mountain layer listening on http://127.0.0.1:{} (mode={:?})",
		port,
		parse_mode()
	);

	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let window_opt = WINDOW.lock().unwrap().clone();
				std::thread::spawn(move || {
					if let Err(e) = handle_connection(&window_opt, &mut stream) {
						eprintln!("[WebkitDebug] Connection error: {}", e);
					}
				});
			}
			Err(e) => eprintln!("[WebkitDebug] Accept error: {}", e),
		}
	}
}

/// Handles a single HTTP connection, dispatches based on method and path.
fn handle_connection(
	window_opt: &Option<Arc<WebviewWindow<Wry>>>,
	stream: &mut std::net::TcpStream,
) -> io::Result<()> {
	// Early check for window initialization
	if window_opt.is_none() {
		send_json(stream, 503, &json!({"error": "debug server not initialized"}))?;
		return Ok(());
	}

	// Read request data (method, path_and_query, body)
	let (method, path_and_query, body) = {
		let mut reader = BufReader::new(&mut *stream);
		let mut request_line = String::new();
		reader.read_line(&mut request_line)?;
		let request_line = request_line.trim_end();
		let parts: Vec<&str> = request_line.split_whitespace().collect();
		if parts.len() != 3 {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "bad request line"));
		}
		let method = parts[0].to_string();
		let path_and_query = parts[1].to_string();

		// Read headers
		let mut headers = HashMap::new();
		loop {
			let mut line = String::new();
			let n = reader.read_line(&mut line)?;
			if n == 0 || line == "\r\n" {
				break;
			}
			if let Some(idx) = line.find(:) {
				let name = line[..idx].trim().to_uppercase();
				let value = line[idx + 1..].trim().to_string();
				headers.insert(name, value);
			}
		}

		// Read body if Content-Length present
		let body = if let Some(len_str) = headers.get("CONTENT-LENGTH") {
			let len: usize = len_str.parse().unwrap_or(0);
			let mut body_bytes = vec![0; len];
			reader.read_exact(&mut body_bytes)?;
			String::from_utf8_lossy(&body_bytes).to_string()
		} else {
			String::new()
		};

		(method, path_and_query, body)
	};

	// Parse URL to get path and query
	let full_url = format!("http://localhost{}", path_and_query);
	let parsed =
		Url::parse(&full_url).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid URL"))?;
	let path = parsed.path();
	let mut query_pairs = parsed.query_pairs();

	// Dispatch request
	let (status, response_json) = match (method.as_str(), path) {
		// ---------- Layer discovery -------------------------------------
		("GET", "/health") => (
			200,
			json!({
				"layer": "mountain",
				"version": env!("CARGO_PKG_VERSION"),
				"pid": std::process::id(),
				"mode": format!("{:?}", parse_mode()),
				"capabilities": [
					"eval","execute","iframes","console","commands",
					"command","vscode/diff","extensions(proxy)","layers"
				],
			}),
		),
		("GET", "/layers") => (
			200,
			json!({
				"mountain": { "enabled": mountain_enabled(parse_mode()), "port": mountain_port() },
				"cocoon":   { "enabled": cocoon_enabled(parse_mode()),   "port": cocoon_port()   },
				"mode": format!("{:?}", parse_mode()),
			}),
		),

		// ---------- Renderer-side primitives ---------------------------
		("GET", "/console") => {
			let js = r#"(function() {
                const logs = window.__MOUNTAIN_DEBUG_CONSOLE || [];
                window.__MOUNTAIN_DEBUG_CONSOLE = [];
                return JSON.stringify(logs);
            })()"#;
			match eval_js(window_opt, js) {
				Ok(value) => (200, json!({"logs": value})),
				Err(e) => (500, json!({"error": e})),
			}
		}
		("GET", "/eval") => {
			let js = query_pairs
				.find(|(k, _)| k == "js")
				.map(|(_, v)| v.into_owned())
				.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing js parameter"))?;
			match eval_js(window_opt, &js) {
				Ok(value) => (200, json!({"result": value})),
				Err(e) => (500, json!({"error": e})),
			}
		}
		("POST", "/execute") => {
			let parsed_body: Value = serde_json::from_str(&body)
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
			let js = parsed_body["js"]
				.as_str()
				.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing js field"))?;
			let target = parsed_body["target"].as_str().unwrap_or("renderer");
			match target {
				"extension-host" | "eh" | "cocoon" => proxy_to_cocoon("POST", "/execute", &body),
				"iframe" => {
					let iframe_id = parsed_body["iframeId"].as_str().unwrap_or("");
					let wrapped = format!(
						r#"(function() {{
                            const ifr = document.querySelector({0});
                            if (!ifr) return JSON.stringify({{ error: "iframe not found" }});
                            try {{
                                return JSON.stringify(ifr.contentWindow.eval({1}));
                            }} catch (e) {{ return JSON.stringify({{ error: String(e) }}); }}
                        }})()"#,
						json!(format!("iframe#{}", iframe_id)),
						json!(js)
					);
					match eval_js(window_opt, &wrapped) {
						Ok(v) => (200, json!({"result": v})),
						Err(e) => (500, json!({"error": e})),
					}
				}
				_ => {
					if js.is_empty() {
						(400, json!({"error": "empty js"}))
					} else {
						match eval_js(window_opt, js) {
							Ok(val) => (200, json!({"result": val})),
							Err(e) => (500, json!({"error": e})),
						}
					}
				}
			}
		}
		("GET", "/iframes") => {
			let js = r#"(function() {
                const frames = document.querySelectorAll(iframe);
                const arr = [];
                frames.forEach(f => {
                    arr.push({
                        src: f.src, id: f.id, name: f.name,
                        contentWindow: !!f.contentWindow
                    });
                });
                return JSON.stringify(arr);
            })()"#;
			match eval_js(window_opt, js) {
				Ok(value) => (200, json!({"iframes": value})),
				Err(e) => (500, json!({"error": e})),
			}
		}

		// ---------- Workbench command surface ---------------------------
		("GET", "/commands") => {
			let js = r#"(async function(){
                try {
                  const r = require(vs/platform/commands/common/commands);
                  const all = r.CommandsRegistry.getCommands();
                  return JSON.stringify(Array.from(all.keys()).slice(0, 5000));
                } catch (e) { return JSON.stringify({error:String(e)}); }
            })()"#;
			match eval_js(window_opt, js) {
				Ok(v) => (200, json!({"commands": v})),
				Err(e) => (500, json!({"error": e})),
			}
		}
		("POST", "/command") => {
			let parsed_body: Value = serde_json::from_str(&body)
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
			let id = parsed_body["id"]
				.as_str()
				.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing id"))?;
			let args = parsed_body.get("args").cloned().unwrap_or_else(|| json!([]));
			let js = format!(
				r#"(async function(){{
                    try {{
                      const cs = require(vs/platform/commands/common/commands).CommandsRegistry;
                      const svcId = require(vs/platform/instantiation/common/instantiation).IInstantiationService;
                      const ws = (globalThis.MonacoEnvironment || globalThis).__workbench__;
                      // Resolve through the workbench command service if available.
                      const cmdSvc = ws?.commandService
                        || ws?.services?.get?.(require(vs/platform/commands/common/commands).ICommandService);
                      if (!cmdSvc) return JSON.stringify({{error:"command service unavailable"}});
                      const args = {0};
                      const result = await cmdSvc.executeCommand({1}, ...args);
                      return JSON.stringify({{ ok: true, result: result ?? null }});
                    }} catch (e) {{ return JSON.stringify({{ ok:false, error: String(e?.stack||e) }}); }}
                }})()"#,
				args, json!(id)
			);
			match eval_js(window_opt, &js) {
				Ok(v) => (200, v),
				Err(e) => (500, json!({"error": e})),
			}
		}
		("POST", "/vscode/diff") => {
			let parsed_body: Value = serde_json::from_str(&body)
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
			let left = parsed_body["left"].as_str().unwrap_or("");
			let right = parsed_body["right"].as_str().unwrap_or("");
			let title = parsed_body["title"].as_str().unwrap_or("Diff");
			if left.is_empty() || right.is_empty() {
				(400, json!({"error":"left and right required"}))
			} else {
				let js = format!(
					r#"(async function(){{
                        try {{
                          const URI = require(vs/base/common/uri).URI;
                          const cmdSvc = (globalThis).__workbench__?.commandService;
                          if (!cmdSvc) return JSON.stringify({{error:"command service unavailable"}});
                          await cmdSvc.executeCommand(vscode.diff, URI.parse({0}), URI.parse({1}), {2});
                          return JSON.stringify({{ok:true}});
                        }} catch (e) {{ return JSON.stringify({{ok:false,error:String(e?.stack||e)}}); }}
                    }})()"#,
					json!(left), json!(right), json!(title)
				);
				match eval_js(window_opt, &js) {
					Ok(v) => (200, v),
					Err(e) => (500, json!({"error": e})),
				}
			}
		}

		// ---------- Cocoon proxy ---------------------------------------
		("GET", "/extensions") => proxy_to_cocoon("GET", "/extensions", ""),

		_ => (404, json!({"error": "not found", "method": method, "path": path})),
	};

	send_json(stream, status, &response_json)
}

/// Evaluates JavaScript in the webview and returns the result as a
/// serde_json::Value.
fn eval_js(window_opt: &Option<Arc<WebviewWindow<Wry>>>, js: &str) -> Result<Value, String> {
	let window = window_opt.as_ref().ok_or("debug server not initialized")?;
	let (tx, rx) = std::sync::mpsc::sync_channel(1);
	window
		.eval_with_callback(js.to_string(), move |result| {
			let _ = tx.send(result);
		})
		.map_err(|e| e.to_string())?;
	let result_str = rx
		.recv_timeout(Duration::from_secs(5))
		.map_err(|_| "timeout waiting for eval result".to_string())?;
	serde_json::from_str(&result_str).map_err(|e| e.to_string())
}

/// Best-effort forward to the Cocoon DebugServer over loopback.
/// Returns (status, json). If Cocoon is unreachable, returns 502.
fn proxy_to_cocoon(method: &str, path: &str, body: &str) -> (u16, Value) {
	use std::net::TcpStream;
	let port = cocoon_port();
	let addr = format!("127.0.0.1:{}", port);
	let mut stream = match TcpStream::connect_timeout(
		&addr.parse().unwrap(),
		Duration::from_millis(300),
	) {
		Ok(s) => s,
		Err(e) => {
			return (
				502,
				json!({"error":"cocoon layer unreachable","detail":e.to_string(),"port":port}),
			)
		}
	};
	let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
	let req = format!(
		"{} {} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
		method,
		path,
		body.len(),
		body
	);
	if stream.write_all(req.as_bytes()).is_err() {
		return (502, json!({"error":"cocoon write failed"}));
	}
	let mut buf = String::new();
	if stream.read_to_string(&mut buf).is_err() {
		return (502, json!({"error":"cocoon read failed"}));
	}
	// Split headers/body.
	let body_idx = buf.find("\r\n\r\n").map(|i| i + 4).unwrap_or(buf.len());
	let body_str = &buf[body_idx..];
	let parsed: Value = serde_json::from_str(body_str).unwrap_or_else(|_| json!({"raw": body_str}));
	(200, parsed)
}

/// Sends a JSON response with the given status code.
fn send_json(stream: &mut std::net::TcpStream, status: u16, value: &Value) -> io::Result<()> {
	let body = serde_json::to_string(value)
		.map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "serialization error"))?;
	let status_text = match status {
		200 => "OK",
		400 => "Bad Request",
		404 => "Not Found",
		500 => "Internal Server Error",
		502 => "Bad Gateway",
		503 => "Service Unavailable",
		_ => "OK",
	};
	let headers = format!(
		"HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
		status,
		status_text,
		body.len()
	);
	stream.write_all(headers.as_bytes())?;
	stream.write_all(body.as_bytes())?;
	stream.flush()?;
	Ok(())
}
