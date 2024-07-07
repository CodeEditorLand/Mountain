#![allow(non_snake_case)]

use Echo::Fn::Job::{Action, ActionResult, Work, Worker};

use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message::Text;

/// Represents a site that holds a WebSocket connection.
///
/// The `Site` struct contains an `Order` field, which is an `Arc` wrapped `Mutex`
/// that protects a WebSocket stream. This allows the WebSocket connection to be
/// safely shared and accessed across multiple asynchronous tasks.
///
struct Site {
	/// The WebSocket connection wrapped in an `Arc` and `Mutex` for safe concurrent access.
	Order: Arc<
		Mutex<
			tokio_tungstenite::WebSocketStream<
				tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
			>,
		>,
	>,
}

/// Implements the `Worker` trait for the `Site` struct, allowing it to process actions.
///
/// This implementation sends actions over a WebSocket connection and waits for a response.
/// The response is then deserialized and returned as an `ActionResult`.
///
/// # Arguments
///
/// * `Action` - The action to be processed.
///
/// # Returns
///
/// An `ActionResult` containing the result of the action.
///
/// # Errors
///
/// Returns an `ActionResult` with an error message if:
/// - Sending the action over the WebSocket connection fails.
/// - Receiving a response from the WebSocket connection fails.
/// - Deserializing the response fails.
///
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

/// Enqueues a write action to the work queue.
///
/// This function is a Tauri command that takes a file path and content, and assigns a `Write` action
/// to the work queue. The action will be processed asynchronously by a worker.
///
/// # Arguments
///
/// * `Path` - A `String` representing the file path to write to.
/// * `Content` - A `String` representing the content to be written to the file.
/// * `Work` - A Tauri state containing an `Arc` reference to a `Work` instance, which holds the queue of actions to be processed.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. Returns `Ok(())` if the action was successfully assigned to the queue.
///
/// # Errors
///
/// This function will return an `Err` if there is an issue assigning the action to the queue.
///
#[tauri::command]
async fn Put(
	Path: String,
	Content: String,
	Work: tauri::State<'_, Arc<Work>>,
) -> Result<(), String> {
	Work.Assign(Action::Write { Path, Content }).await;

	Ok(())
}

/// Enqueues a read action to the work queue.
///
/// This function is a Tauri command that takes a file path and assigns a `Read` action
/// to the work queue. The action will be processed asynchronously by a worker.
///
/// # Arguments
///
/// * `Path` - A `String` representing the file path to read from.
/// * `Work` - A Tauri state containing an `Arc` reference to a `Work` instance, which holds the queue of actions to be processed.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. Returns `Ok(())` if the action was successfully assigned to the queue.
///
/// # Errors
///
/// This function will return an `Err` if there is an issue assigning the action to the queue.
///
#[tauri::command]
async fn Get(Path: String, Work: tauri::State<'_, Arc<Work>>) -> Result<(), String> {
	Work.Assign(Action::Read { Path }).await;

	Ok(())
}

/// Initializes and runs a multi-threaded Tokio runtime, sets up a WebSocket connection,
/// and manages a Tauri application.
///
/// This function performs the following steps:
/// 1. Creates a multi-threaded Tokio runtime.
/// 2. Establishes a WebSocket connection to a specified URL.
/// 3. Initializes a work queue and an approval channel.
/// 4. Spawns multiple worker tasks to process actions from the work queue.
/// 5. Sets up a Tauri application with necessary configurations and plugins.
/// 6. Runs the Tauri application and handles incoming action results.
///
/// # Panics
///
/// This function will panic if:
/// - The Tokio runtime cannot be created.
/// - The WebSocket connection cannot be established.
/// - The Tauri application cannot be run.
///
/// # TODO
///
/// - Auto-calculate the number of workers.
/// - Fix the Tauri plugin setup for development.
/// - Rewrite the emit logic to only emit to a specific webview.
#[allow(dead_code)]
pub fn Fn() {
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
