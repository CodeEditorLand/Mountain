#![allow(non_snake_case)]
//! Cocoon → Mountain `window.createTextEditorDecorationType` /
//! `window.disposeTextEditorDecorationType` notifications. Forwards
//! the payload on `sky://decoration/<suffix>`; Sky's editor renderer
//! owns the Monaco-side decoration lifecycle so Mountain is a pure
//! relay.
//!
//! Per session log audit `20260501T053137`: ~337 create + 317
//! dispose calls, mostly from extension activation registering
//! syntax-highlight decorations for new file types. Each emit hit
//! Tauri's serialised WebKit channel that also delivers keystrokes.
//! 16ms coalescer per method name buffers payloads and emits a
//! single `{ batch: [...] }` per channel per frame; SkyBridge's
//! listener demultiplexes back to per-decoration `cel:decoration:*`
//! CustomEvents.

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

struct DecorationEmitBatch {

	Pending:Mutex<HashMap<String, Vec<Value>>>,

	FlushScheduled:AtomicBool,
}

static DECORATION_EMIT_BATCH:OnceLock<Arc<DecorationEmitBatch>> = OnceLock::new();

fn EnqueueDecorationEmit(Handle:&AppHandle, Channel:String, Payload:Value) {

	let Batch = DECORATION_EMIT_BATCH.get_or_init(|| {
		Arc::new(DecorationEmitBatch { Pending:Mutex::new(HashMap::new()), FlushScheduled:AtomicBool::new(false) })
	});

	{

		let mut Guard = Batch.Pending.lock().unwrap();

		Guard.entry(Channel).or_insert_with(Vec::new).push(Payload);
	}

	if !Batch.FlushScheduled.swap(true, Ordering::AcqRel) {

		let BatchClone = Batch.clone();

		let HandleClone = Handle.clone();

		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_millis(16)).await;
			let Drained:HashMap<String, Vec<Value>> = {
				let mut Guard = BatchClone.Pending.lock().unwrap();
				std::mem::take(&mut *Guard)
			};
			BatchClone.FlushScheduled.store(false, Ordering::Release);
			for (ChannelName, Payloads) in Drained {
				let Count = Payloads.len();
				match HandleClone.emit(&ChannelName, json!({ "batch": Payloads })) {
					Ok(()) => dev_log!("sky-emit", "[SkyEmit] ok channel={} batch={}", ChannelName, Count),
					Err(Error) => {
						dev_log!(
							"sky-emit",

							"[SkyEmit] fail channel={} batch={} error={}",

							ChannelName,

							Count,

							Error
						)
					},
				}
			}
		});
	}
}

pub async fn DecorationTypeLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {

	let EventName = format!("sky://decoration/{}", &MethodName["window.".len()..]);

	EnqueueDecorationEmit(Service.ApplicationHandle(), EventName, Parameter.clone());
}
