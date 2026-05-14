//! Debug webkit inspection server for Mountain.
//! Provides an HTTP endpoint for automated inspection of the WKWebView
//! when launched with DEBUG_SERVER=1 environment variable.
//! The server runs on 127.0.0.1:DEBUG_PORT (default 9933) and exposes:
//! - POST /execute: evaluate JavaScript, return result
//! - GET /console: fetch captured console logs
//! - GET /dom?selector=...: query DOM elements
//! - GET /iframes: list iframes

use std::{
	io::prelude::*,
	net::{TcpListener, TcpStream},
	sync::{Arc, Mutex},
	time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tauri::{ConsoleEvent, Manager, WebviewWindow, Wry};

use crate::dev_log;

#[derive(Clone)]
struct ConsoleLog {
	level:String,
	message:String,
	timestamp:i64,
}

/// Install console log listener and optionally start the debug HTTP server.
/// Call this after the main window has been created.
pub fn install(window:&WebviewWindow<Wry>) {
	#[cfg(debug_assertions)]
	{
		// Capture console logs
		let console_logs = Arc::new(Mutex::new(Vec::<ConsoleLog>::new()));
		let console_logs_clone = console_logs.clone();

		// Attach console log listener to the webview
		if let Err(e) = window.on_console_log(move |event| {
			let level = event.level().to_string();
			let message = event.message().to_string();
			let timestamp = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.map(|d| d.as_secs() as i64)
				.unwrap_or(0);
			let mut logs = console_logs_clone.lock().unwrap();
			logs.push(ConsoleLog { level, message, timestamp });
			if logs.len() > 1000 {
				logs.remove(0);
			}
		}) {
			dev_log!("debug", "[Debug] Failed to install console log listener: {}", e);
		}

		// Check if server should be started
		let enable = std::env::var("DEBUG_SERVER")
			.map(|v| !v.is_empty() && v != "0")
			.unwrap_or(false);
		if enable {
			let window_clone = window.clone();
			let port = std::env::var("DEBUG_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(9933);
			std::thread::spawn(move || {
				dev_log!("debug", "[Debug] Starting WebKit debug server on 127.0.0.1:{}", port);
				match start_debug_server(window_clone, console_logs, port) {
					Ok(_) => dev_log!("debug", "[Debug] WebKit debug server stopped"),
					Err(e) => dev_log!("debug", "[Debug] WebKit debug server error: {}", e),
				}
			});
		}
	}
}

/// Start the debug HTTP server.
fn start_debug_server(
	window:WebviewWindow<Wry>,
	console_logs:Arc<Mutex<Vec<ConsoleLog>>>,
	port:u16,
) -> std::io::Result<()> {
	let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
	listener.set_nonblocking(false)?;
	dev_log!("debug", "[Debug] WebKit inspector server listening on 127.0.0.1:{}", port);
	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				let window = window.clone();
				let console_logs = console_logs.clone();
				std::thread::spawn(move || {
					let _ = handle_connection(&mut stream, &window, &console_logs);
				});
			},
			Err(e) => {
				eprintln!("Debug server accept error: {}", e);
			},
		}
	}
	Ok(())
}

/// Handle a single HTTP connection.
fn handle_connection(
	stream:&mut TcpStream,
	window:&WebviewWindow<Wry>,
	console_logs:&Arc<Mutex<Vec<ConsoleLog>>>,
) -> std::io::Result<()> {
	// Read entire request into buffer (client closes after request)
	let mut buffer = Vec::new();
	let mut temp = [0; 16384];
	loop {
		let n = stream.read(&mut temp)?;
		if n == 0 {
			break;
		}
		buffer.extend_from_slice(&temp[..n]);
	}

	// Split headers and body using double CRLF
	let body_start = buffer.windows(4).position(|w| w == b"\r\n\r\n").map(|idx| idx + 4);
	let body_bytes = if let Some(start) = body_start { &buffer[start..] } else { &[] };
	let body_str = String::from_utf8_lossy(body_bytes);

	// Parse request line
	let request_str = String::from_utf8_lossy(&buffer);
	let lines:Vec<&str> = request_str.lines().collect();
	let first_line = lines.first().unwrap_or(&"");
	let parts:Vec<&str> = first_line.split_whitespace().collect();
	if parts.len() < 2 {
		let resp = b"HTTP/1.1 400 Bad Request\r\n\r\n";
		stream.write_all(resp)?;
		return Ok(());
	}
	let method = parts[0];
	let path = parts[1];

	// Helper to send JSON response
	let send_json = |status:u16, data:Value| -> std::io::Result<()> {
		let body = data.to_string();
		let reason = if status == 200 { "OK" } else { "Not Found" };
		let response = format!(
			"HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
			status,
			reason,
			body.len(),
			body
		);
		stream.write_all(response.as_bytes())?;
		stream.flush()?;
		Ok(())
	};

	let response = if method == "GET" && path == "/health" {
		json!({"status":"ok"})
	} else if method == "GET" && path == "/console" {
		let logs = console_logs.lock().unwrap().clone();
		let logs_ser:Vec<Value> = logs
			.iter()
			.map(|l| json!({"level": l.level, "message": l.message, "timestamp": l.timestamp}))
			.collect();
		json!({"logs": logs_ser})
	} else if method == "GET" && path.starts_with("/dom?") {
		// Very simple query parse
		let query = &path["/dom?".len()..];
		let selector = query
			.split('&')
			.find(|s| s.starts_with("selector="))
			.map(|s| &s["selector=".len()..])
			.unwrap_or("*");
		// Basic escaping for single quotes
		let selector_escaped = selector.replace("\\", "\\\\").replace("'", "\\'");
		let js = format!(
			r#"
            (function() {{
                const results = Array.from(document.querySelectorAll('{}'));
                return results.map(el => {{
                    return {{
                        tagName: el.tagName,
                        id: el.id,
                        classes: Array.from(el.classList),
                        attributes: (function() {{
                            const attrs = {{}};
                            for (let attr of el.attributes) {{
                                attrs[attr.name] = attr.value;
                            }}
                            return attrs;
                        }})(),
                        text: (el.textContent || '').trim().slice(0, 200)
                    }};
                }});
            }})()
            "#,
			selector_escaped
		);
		match window.eval_script(&js) {
			Ok(value) => json!({"elements": value}),
			Err(e) => json!({"error": e.to_string(), "elements": json!([])}),
		}
	} else if method == "GET" && path == "/iframes" {
		let js = r#"
            (function() {
                const iframes = Array.from(document.querySelectorAll('iframe'));
                return iframes.map(f => {
                    let depth = 0;
                    let el = f.parentElement;
                    while (el && el.tagName !== 'BODY') { depth++; el = el.parentElement; }
                    return {
                        id: f.id || '',
                        name: f.name || '',
                        src: f.src || '',
                        depth: depth
                    };
                });
            })()
        "#;
		match window.eval_script(js) {
			Ok(value) => json!({"iframes": value}),
			Err(e) => json!({"error": e.to_string(), "iframes": json!([])}),
		}
	} else if method == "POST" && path == "/execute" {
		let body_json:Value = serde_json::from_str(&body_str).unwrap_or_default();
		let js = body_json["js"].as_str().unwrap_or("");
		if js.is_empty() {
			return send_json(400, json!({"error":"missing js field","result":null}));
		}
		match window.eval_script(js) {
			Ok(value) => send_json(200, json!({"result": value, "error": null})),
			Err(e) => send_json(200, json!({"error": e.to_string(), "result": null})),
		}
	} else {
		send_json(404, json!({"error": "not found"}))
	};

	match response {
		Ok(_) => Ok(()),
		Err(e) => {
			eprintln!("Debug server response error: {}", e);
			Ok(())
		},
	}
}
