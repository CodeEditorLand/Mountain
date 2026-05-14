//! Debug server for inspecting webview content via HTTP API.
//! Only compiled in debug builds.

use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    time::Duration,
};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use tauri::{WebviewWindow, Wry};
use url::Url;

/// Global storage for the webview window used by the debug server.
static WINDOW: Lazy<Mutex<Option<Arc<WebviewWindow<Wry>>>>> = Lazy::new(|| Mutex::new(None));

/// Installs the debug server and stores a reference to the webview window.
/// This function should be called once during app setup in debug builds.
pub fn install(window: &WebviewWindow<Wry>) {
    // Store the window for later use
    let mut guard = WINDOW.lock().unwrap();
    *guard = Some(Arc::new(window.clone()));

    // Start the HTTP server in a background thread if enabled
    let enable = std::env::var("DEBUG_SERVER")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if enable {
        std::thread::spawn(|| {
            start_server();
        });
    }
}

/// Main server loop listening for TCP connections.
fn start_server() {
    let port = std::env::var("DEBUG_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9933);

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[WebkitDebug] Failed to bind to 127.0.0.1:{}: {}", port, e);
            return;
        }
    };
    eprintln!("[WebkitDebug] Server listening on http://127.0.0.1:{}", port);

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
    let (method, path_and_query, body, malformed) = {
        let mut reader = BufReader::new(&mut *stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;
        let request_line = request_line.trim_end();
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() != 3 {
            // Malformed request line; return early after dropping reader
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
            if let Some(idx) = line.find(':') {
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

        (method, path_and_query, body, false)
    };

    // Parse URL to get path and query
    let full_url = format!("http://localhost{}", path_and_query);
    let parsed = Url::parse(&full_url).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid URL"))?;
    let path = parsed.path();
    let mut query_pairs = parsed.query_pairs();

    // Dispatch request
    let (status, response_json) = match method.as_str() {
        "GET" if path == "/console" => {
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
        "GET" if path == "/eval" => {
            let js = query_pairs
                .find(|(k, _)| k == "js")
                .map(|(_, v)| v.into_owned())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing js parameter"))?;
            match eval_js(window_opt, &js) {
                Ok(value) => (200, json!({"result": value})),
                Err(e) => (500, json!({"error": e})),
            }
        }
        "POST" if path == "/execute" => {
            let parsed_body: Value = serde_json::from_str(&body)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            let js = parsed_body["js"]
                .as_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing js field"))?;
            if js.is_empty() {
                (400, json!({"error": "empty js"}))
            } else {
                match eval_js(window_opt, js) {
                    Ok(val) => (200, json!({"result": val})),
                    Err(e) => (500, json!({"error": e})),
                }
            }
        }
        "GET" if path == "/iframes" => {
            let js = r#"(function() {
                const frames = document.querySelectorAll('iframe');
                const arr = [];
                frames.forEach(f => {
                    arr.push({
                        src: f.src,
                        id: f.id,
                        name: f.name,
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
        _ => (404, json!({"error": "not found"})),
    };

    send_json(stream, status, &response_json)
}

/// Evaluates JavaScript in the webview and returns the result as a serde_json::Value.
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

/// Sends a JSON response with the given status code.
fn send_json(
    stream: &mut std::net::TcpStream,
    status: u16,
    value: &Value,
) -> io::Result<()> {
    let body = serde_json::to_string(value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "serialization error"))?;
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
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
