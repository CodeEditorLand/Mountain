#![allow(non_snake_case)]

//! Cocoon → Mountain `window.setTextEditorDecorations` notification.
//!
//! Fired whenever an extension calls `editor.setDecorations(decorationType,
//! rangesOrOptions)`. Mountain forwards the payload as
//! `sky://decoration/set-ranges` so Sky's ICodeEditorService can apply the
//! ranges to the Monaco editor for the matching URI.
//!
//! Payload shape (from Cocoon `Window/Namespace.ts`):
//! ```json
//! {
//!   "decorationTypeKey": "GitLens.blame",
//!   "uri": "file:///path/to/file.ts",
//!   "rangesOrOptions": [
//!     { "range": { "startLineNumber": 1, "startColumn": 1, "endLineNumber": 1, "endColumn": 80 } }
//!   ]
//! }
//! ```
//!
//! Channel-drain batching: ~5-200 calls per extension per second during
//! scroll; one Tauri event per frame (16 ms window, drain stragglers).

use std::sync::OnceLock;

use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

struct DecoSetItem {
	Handle:AppHandle,
	Payload:Value,
}

struct DecoSetChannel {
	Sender:UnboundedSender<DecoSetItem>,
}

static DECO_SET_CH:OnceLock<DecoSetChannel> = OnceLock::new();

fn GetOrInitChannel(Handle:&AppHandle) -> &'static DecoSetChannel {
	DECO_SET_CH.get_or_init(|| {
		let (Tx, mut Rx) = unbounded_channel::<DecoSetItem>();

		tokio::spawn(async move {
			let mut Buf:Vec<DecoSetItem> = Vec::with_capacity(64);

			loop {
				match Rx.recv().await {
					None => break,
					Some(Item) => Buf.push(Item),
				}

				// Drain stragglers within one animation frame
				Rx.recv_many(&mut Buf, 4096).await;
				tokio::time::sleep(std::time::Duration::from_millis(16)).await;
				Rx.recv_many(&mut Buf, 4096).await;

				if Buf.is_empty() {
					continue;
				}

				// Batch all queued set-ranges calls into one Tauri event
				let Handle = Buf[0].Handle.clone();
				let Batch:Vec<Value> = Buf.drain(..).map(|I| I.Payload).collect();

				match Handle.emit("sky://decoration/set-ranges", serde_json::json!({ "batch": Batch })) {
					Ok(()) => dev_log!("sky-emit", "[DecoSet] emitted batch={}", Batch.len()),
					Err(E) => dev_log!("sky-emit", "[DecoSet] emit failed: {}", E),
				}
			}
		});

		DecoSetChannel { Sender:Tx }
	})
}

pub async fn SetTextEditorDecorations(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Ch = GetOrInitChannel(Service.ApplicationHandle());
	let _ = Ch
		.Sender
		.send(DecoSetItem { Handle:Service.ApplicationHandle().clone(), Payload:Parameter.clone() });
}
