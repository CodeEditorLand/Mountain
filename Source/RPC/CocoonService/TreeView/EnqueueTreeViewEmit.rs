#![allow(non_snake_case)]

//! Coalesce 30+ Mountain → Sky `tree-view/create` emits at boot into a
//! single batched payload per 16 ms window. SkyBridge's listener accepts
//! both single `{ viewId, extensionId }` and batch `{ views: [...] }`
//! shapes (mirrors the command-batch pattern from
//! `Vine/Server/Notification/RegisterCommand.rs`).

use std::{
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
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::dev_log;

struct Batch {
	Pending:Mutex<Vec<Value>>,
	FlushScheduled:AtomicBool,
}

static BATCH:OnceLock<Arc<Batch>> = OnceLock::new();

pub fn Fn(Handle:&AppHandle, Payload:Value) {
	let Batch =
		BATCH.get_or_init(|| Arc::new(Batch { Pending:Mutex::new(Vec::new()), FlushScheduled:AtomicBool::new(false) }));

	{
		let mut Pending = Batch.Pending.lock().unwrap();
		Pending.push(Payload);
	}

	if !Batch.FlushScheduled.swap(true, Ordering::AcqRel) {
		let Cloned = Batch.clone();
		let HandleCloned = Handle.clone();
		let Channel = SkyEvent::TreeViewCreate.AsStr().to_string();
		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_millis(16)).await;
			let Drained:Vec<Value> = {
				let mut Pending = Cloned.Pending.lock().unwrap();
				std::mem::take(&mut *Pending)
			};
			Cloned.FlushScheduled.store(false, Ordering::Release);
			if Drained.is_empty() {
				return;
			}
			let Count = Drained.len();
			match HandleCloned.emit(&Channel, json!({ "views": Drained })) {
				Ok(()) => dev_log!("sky-emit", "[SkyEmit] ok channel={} batch={}", Channel, Count),
				Err(Error) => {
					dev_log!("sky-emit", "[SkyEmit] fail channel={} batch={} error={}", Channel, Count, Error)
				},
			}
		});
	}
}
