#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use Echo::Fn::Job::{Action, ActionResult, Work, Worker};

use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message::Text;

struct Site {
	Order: Arc<
		Mutex<
			tokio_tungstenite::WebSocketStream<
				tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
			>,
		>,
	>,
}

#[async_trait::async_trait]
impl Worker for Site {
	async fn Receive(&self, Action: Action) -> ActionResult {
		let mut Order = self.Order.lock().await;

		if Order.send(Text(serde_json::to_string(&Action).unwrap())).await.is_err() {
			return ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) };
		}

		if let Some(response) = Order.next().await {
			match response {
				Ok(Text(text)) => serde_json::from_str(&text).unwrap_or(ActionResult {
					Action,
					Result: Err("Cannot serde_json.".to_string()),
				}),
				_ => ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) },
			}
		} else {
			ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) }
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
pub async fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot new_multi_thread.")
		.block_on(async {
			let Order = Arc::new(Mutex::new(
				tokio_tungstenite::connect_async("ws://localhost:9999")
					.await
					.expect("Cannot connect_async.")
					.0,
			));

			let Work = Arc::new(Work::Begin());
			let (Approval, mut Receipt) = tokio::sync::mpsc::unbounded_channel();

			// @TODO: Auto-calc number of workers on the force
			let Force: Vec<_> = (0..4)
				.map(|_| {
					tokio::spawn(Echo::Fn::Job::Fn(
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
				.setup(|Tauri| {
					let Handle = Tauri.handle().clone();

					tokio::spawn(async move {
						while let Some(ActionResult) = Receipt.recv().await {
							// @TODO: Rewrite the Emit to only emit to a specific webview which then talks to the others
							Handle.emit("ActionResult", ActionResult).unwrap();
						}
					});

					Ok(())
				})
				.manage(Work)
				.invoke_handler(tauri::generate_handler![Put, Get])
				.plugin(tauri_plugin_shell::init())
				.run(tauri::generate_context!())
				.expect("Cannot Library.");

			futures::future::join_all(Force).await;
		});
}
