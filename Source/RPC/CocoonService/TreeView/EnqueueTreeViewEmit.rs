//! Coalesce 30+ Mountain → Sky `tree-view/create` emits at boot into a
//! single batched payload per frame. Uses the channel-drain pattern: a
//! long-lived flusher wakes on first item, drains immediately, sleeps one
//! frame (16 ms), drains stragglers, then emits one `{ views: [...] }` batch.
//! Zero spawns per call; sub-millisecond wake latency.
use std::sync::OnceLock;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::dev_log;

struct TreeViewChannel {
	Sender:UnboundedSender<(AppHandle, Value)>,
}

static TV_CH:OnceLock<TreeViewChannel> = OnceLock::new();

fn GetOrInitChannel(Handle:&AppHandle) -> &'static TreeViewChannel {
	TV_CH.get_or_init(|| {
		let (Tx, mut Rx) = unbounded_channel::<(AppHandle, Value)>();

		let Channel = SkyEvent::TreeViewCreate.AsStr().to_string();

		tokio::spawn(async move {
			let mut Buf:Vec<(AppHandle, Value)> = Vec::with_capacity(64);

			loop {
				match Rx.recv().await {
					None => break,
					Some(Item) => Buf.push(Item),
				}

				Rx.recv_many(&mut Buf, 4096).await;

				tokio::time::sleep(std::time::Duration::from_millis(16)).await;

				Rx.recv_many(&mut Buf, 4096).await;

				match Buf.is_empty() {
					true => continue,
					false => {},
				}

				let Handle = Buf[0].0.clone();

				let Views:Vec<Value> = Buf.drain(..).map(|(_, V)| V).collect();

				let Count = Views.len();

				match Handle.emit(&Channel, json!({ "views": Views })) {
					Ok(()) => dev_log!("sky-emit", "[SkyEmit] ok channel={} batch={}", Channel, Count),
					Err(E) => {
						dev_log!("sky-emit", "[SkyEmit] fail channel={} batch={} error={}", Channel, Count, E)
					},
				}
			}
		});

		TreeViewChannel { Sender:Tx }
	})
}

pub fn Fn(Handle:&AppHandle, Payload:Value) {
	let Ch = GetOrInitChannel(Handle);

	let _ = Ch.Sender.send((Handle.clone(), Payload));
}
