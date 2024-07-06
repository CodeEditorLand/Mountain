#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// @TODO: Finish this and import proper common libs from echo
use echo::{Action, ActionResult, Job, WorkQueue, Worker, Yell};

use async_trait::async_trait;
use futures::{future::join_all, SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::{
	net::TcpStream,
	sync::{mpsc, Mutex},
};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream};

struct Site {
	Order: Arc<Mutex<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

// #[async_trait]
// trait Worker: Send + Sync {
// 	async fn Receive(&self, Action: String) -> Result<String, String>;
// }

#[async_trait]
impl Worker for Site {
	async fn Receive(&self, Action: Action) -> Result<String, String> {
		let mut Order = self.Order.lock().await;

		Order.send(Message::Text(Action)).await.map_err(|Error| Error.to_string())?;

		if let Some(Next) = Order.next().await {
			match Next {
				Ok(Message::Text(Order)) => Ok(Order),
				_ => Err("Cannot Next.".to_string()),
			}
		} else {
			Err("Cannot Order.".to_string())
		}
	}
}

struct Work {
	Queue: Arc<Mutex<Vec<String>>>,
}

impl Work {
	fn Begin() -> Self {
		Work { Queue: Arc::new(Mutex::new(Vec::new())) }
	}

	async fn Assign(&self, Action: String) {
		self.Queue.lock().await.push(Action);
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
async fn Get(Path: String, State: tauri::State<'_, Arc<Work>>) -> Result<(), String> {
	State
		.Assign(
			serde_json::json!({
				"Action": "Get",
				"Path": Path,
			})
			.to_string(),
		)
		.await;
	Ok(())
}

async fn Job(Worker: Arc<dyn Worker>, Work: Arc<Work>, Hire: mpsc::Sender<String>) {
	loop {
		if let Some(Action) = Work.Execute().await {
			match Worker.Receive(Action).await {
				Ok(Result) => {
					if Hire.send(Result).await.is_err() {
						break;
					}
				}

				Err(Error) => eprintln!("Cannot Receive: {}", Error),
			}
		} else {
			tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		}
	}
}

#[allow(dead_code)]
pub async fn Fn() {
	let Order = Arc::new(Mutex::new(
		tokio_tungstenite::connect_async("ws://localhost:9999")
			.await
			.expect("Cannot connect_async.")
			.0,
	));

	let Work = Arc::new(Work::Begin());
	let (Hire, mut Receipt) = mpsc::channel(100);

	// @TODO: Auto-calc number of workers in the force
	let Force: Vec<_> = (0..4)
		.map(|_| {
			tokio::spawn(Job(
				Arc::new(Site { Order: Order.clone() }) as Arc<dyn Worker>,
				Work.clone(),
				Hire.clone(),
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
}
