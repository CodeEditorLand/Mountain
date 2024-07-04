#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use async_trait::async_trait;
use futures::future::join_all;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::{
	net::TcpStream,
	sync::{mpsc, Mutex},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[async_trait]
trait Worker: Send + Sync {
	async fn Receive(&self, Task: String) -> Result<String, String>;
}

struct FileOpWorker {
	Stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>,
}

#[async_trait]
impl Worker for FileOpWorker {
	async fn Receive(&self, Task: String) -> Result<String, String> {
		let mut Stream = self.Stream.lock().await;

		Stream.send(Message::Text(Task)).await.map_err(|Error| Error.to_string())?;

		if let Some(Response) = Stream.next().await {
			match Response {
				Ok(Message::Text(Text)) => Ok(Text),
				_ => Err("Cannot Response.".to_string()),
			}
		} else {
			Err("Cannot Response.".to_string())
		}
	}
}

struct Work {
	Queue: Arc<Mutex<Vec<String>>>,
}

impl Work {
	fn new() -> Self {
		Work { Queue: Arc::new(Mutex::new(Vec::new())) }
	}

	async fn Assign(&self, Task: String) {
		self.Queue.lock().await.push(Task);
	}

	async fn Execute(&self) -> Option<String> {
		self.Queue.lock().await.pop()
	}
}

#[tauri::command]
async fn Put(
	Path: String,
	Content: String,
	State: tauri::State<'_, Arc<Work>>,
) -> Result<(), String> {
	State
		.Assign(
			serde_json::json!({
				"Action": "Put",
				"Path": Path,
				"Content": Content,
			})
			.to_string(),
		)
		.await;

	Ok(())
}

#[tauri::command]
async fn Get(Path: String, State: tauri::State<'_, Arc<WorkQueue>>) -> Result<(), String> {
	State
		.push(
			serde_json::json!({
				"Action": "Get",
				"Path": Path,
			})
			.to_string(),
		)
		.await;
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
