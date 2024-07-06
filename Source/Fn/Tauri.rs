#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// @TODO: Finish this and import proper common libs from echo
use Echo::Fn::Job::{Action, ActionResult, Fn as Job, Work, Worker, Yell::Fn as Yell};

use async_trait::async_trait;
use futures::{future::join_all, SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::{
	net::TcpStream,
	sync::{mpsc, Mutex},
};
use tokio_tungstenite::{
	tungstenite::{protocol::Message, Message::Text},
	MaybeTlsStream,
};

use serde_json::json;

struct Site {
	Order: Arc<Mutex<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

#[async_trait]
impl Worker for Site {
	async fn Receive(&self, Action: Action) -> ActionResult {
		let mut Order = self.Order.lock().await;

		if Order.send(Text(serde_json::to_string(&Action).unwrap())).await.is_err() {
			return ActionResult { Action, Result: Err("Failed to send message".to_string()) };
		}

		if let Some(response) = Order.next().await {
			match response {
				Ok(Text(text)) => serde_json::from_str(&text).unwrap_or(ActionResult {
					Action,
					Result: Err("Cannot serde_json.".to_string()),
				}),
				_ => ActionResult { Action, Result: Err("Cannot Result.".to_string()) },
			}
		} else {
			ActionResult { Action, Result: Err("Cannot Result.".to_string()) }
		}
	}
}

#[tauri::command]
async fn Put(
	Path: String,
	Content: String,
	Work: tauri::State<'_, Arc<Work>>,
) -> Result<(), String> {
	Work.Assign(Action::Write { Path, Content }).await;

	Ok(())
}

#[tauri::command]
async fn Get(Path: String, Work: tauri::State<'_, Arc<Work>>) -> Result<(), String> {
	Work.Assign(Action::Read { Path }).await;

	Ok(())
}

#[allow(dead_code)]
#[tokio::main]
pub async fn Fn() {
	let Order = Arc::new(tokio::sync::Mutex::new(
		tokio_tungstenite::connect_async("ws://localhost:9999")
			.await
			.expect("Cannot connect_async.")
			.0,
	));

	let Work = Arc::new(Work::Begin());
	let (Approval, mut Receipt) = mpsc::channel(100);

	// @TODO: Auto-calc number of workers in the force
	let Force: Vec<_> = (0..4)
		.map(|_| {
			tokio::spawn(Job(
				Arc::new(Site { Order: Order.clone() }) as Arc<dyn Worker>,
				Work.clone(),
				Approval.clone(),
			))
		})
		.collect();

	let Builder = tauri::Builder::default();

	// @TODO: FIX THIS
	// #[cfg(debug_assertions)]
	// Builder.plugin(tauri_plugin_devtools::init());

	Builder
		.setup(|app| {
			let Handle = app.handle().clone();

			tokio::spawn(async move {
				while let Some(ActionResult) = Receipt.recv().await {
					// @TODO: Rewrite the Emit to only emit to a specific webview which then talks to the others
					Handle.emit("file_operation_result", ActionResult).unwrap();
				}
			});

			Ok(())
		})
		.manage(Work)
		.invoke_handler(tauri::generate_handler![Put, Get])
		.plugin(tauri_plugin_shell::init())
		.run(tauri::generate_context!())
		.expect("Cannot Library.");

	join_all(Force).await;

	// @TODO: Introduce a tokio::runtime instead of tokio::main
	// let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
	//  runtime.block_on(async {
	//  ...
	// 	join_all(Force).await;
	// });
}
