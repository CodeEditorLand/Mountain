#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.report` notification.
//! Fires on every `Progress.report({ message, increment })` callback
//! within a `vscode.window.withProgress(...)` run. The git extension
//! alone fires 6000+ of these per session during repository scans;
//! emitting one Tauri event per call saturates the WKWebView IPC
//! channel that also delivers keystrokes. Each event is coalesced
//! into a 16ms (one frame) window per Progress handle, accumulating
//! `increment` deltas and keeping the most recent non-empty
//! `message`. Sky sees one update per frame per progress operation
//! instead of dozens, with the same final cumulative state.

use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex,
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

use serde_json::{Value, json};

use tauri::{AppHandle, Emitter};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

#[derive(Default)]
struct ProgressAccumulator {

	Message:String,

	Increment:f64,
}

struct ProgressEmitBatch {

	Pending:Mutex<HashMap<String, ProgressAccumulator>>,

	FlushScheduled:AtomicBool,
}

static PROGRESS_EMIT_BATCH:OnceLock<Arc<ProgressEmitBatch>> = OnceLock::new();

fn EnqueueProgressEmit(Handle:&AppHandle, ProgressHandle:String, Message:String, Increment:f64) {

	let Batch = PROGRESS_EMIT_BATCH.get_or_init(|| {
		Arc::new(ProgressEmitBatch { Pending:Mutex::new(HashMap::new()), FlushScheduled:AtomicBool::new(false) })
	});

	{

		let mut Guard = Batch.Pending.lock().unwrap();

		let Entry = Guard.entry(ProgressHandle).or_default();

		// VS Code semantics: `message` replaces (latest wins); empty
		// message means "keep previous". `increment` is per-call delta;
		// accumulate so the final emit carries the same total movement.
		if !Message.is_empty() {

			Entry.Message = Message;
		}

		Entry.Increment += Increment;
	}

	if !Batch.FlushScheduled.swap(true, Ordering::AcqRel) {

		let BatchClone = Batch.clone();

		let HandleClone = Handle.clone();

		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_millis(16)).await;
			let Drained:HashMap<String, ProgressAccumulator> = {
				let mut Guard = BatchClone.Pending.lock().unwrap();
				std::mem::take(&mut *Guard)
			};
			BatchClone.FlushScheduled.store(false, Ordering::Release);
			for (ProgressHandleId, Accumulator) in Drained {
				if let Err(Error) = HandleClone.emit(
					"sky://notification/progress-update",

					json!({
						"id": ProgressHandleId,
						"message": Accumulator.Message,
						"increment": Accumulator.Increment,
					}),
				) {
					dev_log!(
						"grpc",

						"warn: [MountainVinegRPCService] sky://notification/progress-update emit failed: {}",

						Error
					);
				}
			}
		});
	}
}

pub async fn ProgressReport(Service:&MountainVinegRPCService, Parameter:&Value) {

	let ProgressHandle = Parameter.get("handle").and_then(Value::as_str).unwrap_or("").to_string();

	let Message = Parameter.get("message").and_then(Value::as_str).unwrap_or("").to_string();

	let Increment = Parameter.get("increment").and_then(Value::as_f64).unwrap_or(0.0);

	EnqueueProgressEmit(Service.ApplicationHandle(), ProgressHandle, Message, Increment);
}
