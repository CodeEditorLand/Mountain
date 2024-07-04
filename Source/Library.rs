#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone, serde::Serialize)]
struct Payload {
	Message: String,
}

#[tauri::command]
async fn Put(
	Path: String,
	Content: String,
	State: tauri::State<'_, Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>>,
) -> Result<(), String> {
	State
		.lock()
		.await
		.send(Message::Text(
			serde_json::json!({
				"Action": "Put",
				"Path": Path,
				"Content": Content,
			})
			.to_string(),
		))
		.await
		.map_err(|e| e.to_string())?;

	Ok(())
}

#[tauri::command]
async fn Get(
	Path: String,
	State: tauri::State<'_, Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>>,
) -> Result<(), String> {
	State
		.lock()
		.await
		.send(Message::Text(
			serde_json::json!({
				"Action": "Get",
				"Path": Path,
			})
			.to_string(),
		))
		.await
		.map_err(|e| e.to_string())?;

	Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(dead_code)]
#[tokio::main]
async fn main() {
	let (Stream, _) = connect_async("ws://localhost:8080").await.expect("Cannot connect_async.");

	let Stream = Arc::new(Mutex::new(Stream));

	let Builder = tauri::Builder::default();

	// @TODO: FIX THIS
	// #[cfg(debug_assertions)]

	// Builder.plugin(tauri_plugin_devtools::init());

	Builder
		.setup(|app| {
			let Handle = app.app_handle();
			let Stream = Stream.clone();

			tokio::spawn(async move {
				while let Some(Message) = Stream.lock().await.next().await {
					if let Ok(msg) = Message {
						if let Message::Text(Text) = msg {
							// @TODO: Rewrite the Emit to only emit to a specific webview which then talks to the others
							Handle.emit("file_content", Payload { Message: Text }).unwrap();
						}
					}
				}
			});

			Ok(())
		})
		.manage(Stream)
		.invoke_handler(tauri::generate_handler![Put, Get])
		.plugin(tauri_plugin_shell::init())
		.run(tauri::generate_context!())
		.expect("Cannot Library.");
}
