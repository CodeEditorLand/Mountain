#![allow(non_snake_case)]

pub mod Connect;
pub mod Get;

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

			// TODO: Auto-calc number of workers on the force
			let Force: Vec<_> = (0..4)
				.map(|_| {
					tokio::spawn(Echo::Fn::Job::Fn(
						Arc::new(crate::Struct::Binary::Site { Order: Order.clone() }) as Arc<dyn Worker>,
						Work.clone(),
						Approval.clone(),
					))
				})
				.collect();

			let Builder = tauri::Builder::default();

			// TODO: FIX THIS
			// #[cfg(debug_assertions)]
			// Builder.plugin(tauri_plugin_devtools::init());

			Builder
				.setup(|Tauri| {
					let Handle = Tauri.handle().clone();

					tokio::spawn(async move {
						while let Some(ActionResult) = Receipt.recv().await {
							// TODO: Rewrite the Emit to only emit to a specific webview which then corresponds with the rest
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

use Echo::Fn::Job::{Action, ActionResult, Work, Worker};

use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message::Text, MaybeTlsStream, WebSocketStream};
