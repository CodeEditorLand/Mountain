//! Cocoon → Mountain `progress.report` notification.
//!
//! The git extension alone fires 6000+ of these per session. We push into
//! an `mpsc::unbounded_channel`; a single long-lived flusher task wakes on
//! the first item, drains everything queued, sleeps 16 ms (one frame), drains
//! again, then emits one batched Tauri event per progress handle with the
//! accumulated `increment` and latest non-empty `message`. Zero spawns per
//! call; sub-millisecond first-wake; single event per handle per frame.

use std::sync::OnceLock;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

struct ProgressItem {
	Handle:AppHandle,

	ProgressHandle:String,

	Message:String,

	Increment:f64,
}

struct ProgressChannel {
	Sender:UnboundedSender<ProgressItem>,
}

static PROGRESS_CH:OnceLock<ProgressChannel> = OnceLock::new();

fn GetOrInitChannel(Handle:&AppHandle) -> &'static ProgressChannel {
	PROGRESS_CH.get_or_init(|| {
		let (Tx, mut Rx) = unbounded_channel::<ProgressItem>();

		tokio::spawn(async move {
			let mut Buf:Vec<ProgressItem> = Vec::with_capacity(64);

			loop {
				match Rx.recv().await {
					None => break,
					Some(Item) => Buf.push(Item),
				}

				Rx.recv_many(&mut Buf, 4096).await;

				tokio::time::sleep(std::time::Duration::from_millis(16)).await;

				Rx.recv_many(&mut Buf, 4096).await;

				if Buf.is_empty() {
					continue;
				}

				// Merge per-handle: latest non-empty message, summed increments.
				let mut ByHandle:std::collections::HashMap<String, (AppHandle, String, f64)> =
					std::collections::HashMap::new();

				for Item in Buf.drain(..) {
					let Entry = ByHandle
						.entry(Item.ProgressHandle.clone())
						.or_insert_with(|| (Item.Handle.clone(), String::new(), 0.0));

					if !Item.Message.is_empty() {
						Entry.1 = Item.Message;
					}

					Entry.2 += Item.Increment;
				}

				for (ProgressHandleId, (AppHandle, Message, Increment)) in ByHandle {
					if let Err(E) = AppHandle.emit(
						"sky://notification/progress-update",
						json!({
							"id": ProgressHandleId,
							"message": Message,
							"increment": Increment,
						}),
					) {
						dev_log!(
							"grpc",
							"warn: [ProgressReport] emit failed handle={} error={}",
							ProgressHandleId,
							E
						);
					}
				}
			}
		});

		ProgressChannel { Sender:Tx }
	})
}

pub async fn ProgressReport(Service:&MountainVinegRPCService, Parameter:&Value) {
	let ProgressHandle = Parameter.get("handle").and_then(Value::as_str).unwrap_or("").to_string();

	let Message = Parameter.get("message").and_then(Value::as_str).unwrap_or("").to_string();

	let Increment = Parameter.get("increment").and_then(Value::as_f64).unwrap_or(0.0);

	let Ch = GetOrInitChannel(Service.ApplicationHandle());

	let _ =
		Ch.Sender
			.send(ProgressItem { Handle:Service.ApplicationHandle().clone(), ProgressHandle, Message, Increment });
}
