//! Cocoon → Mountain `registerCommand` notification.
//! Stores the command as a `Proxied` handler in Mountain's
//! `CommandRegistry` so subsequent `commands.executeCommand` calls get
//! routed back to Cocoon via `$executeContributedCommand` gRPC.
//!
//! ## Batching
//!
//! Extension boot fires 1000+ `registerCommand` notifications in a tight
//! burst. Rather than spawning one short-lived tokio task per call (and
//! always sleeping 16 ms even for the last item), we push into a
//! `mpsc::unbounded_channel` and a single long-lived flusher task drains
//! it: it wakes immediately when the first item arrives, collects
//! everything already queued via `recv_many`, then sleeps 16 ms and
//! drains a second time to catch stragglers - then emits one batch event.
//! The net effect is identical to the old coalescer but avoids 1000+
//! task spawns and reduces the minimum latency to sub-millisecond for
//! isolated commands registered after boot.

use std::sync::OnceLock;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::{
	Environment::CommandProvider::CommandHandler,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

struct CommandBatchChannel {
	Sender:UnboundedSender<(AppHandle, Value)>,
}

static CMD_CHANNEL:OnceLock<CommandBatchChannel> = OnceLock::new();

fn GetOrInitChannel(Handle:&AppHandle) -> &'static CommandBatchChannel {
	CMD_CHANNEL.get_or_init(|| {
		let (Tx, mut Rx) = unbounded_channel::<(AppHandle, Value)>();

		tokio::spawn(async move {
			let mut Buf:Vec<(AppHandle, Value)> = Vec::with_capacity(128);

			loop {
				// Block until at least one item arrives.
				match Rx.recv().await {
					None => break,
					Some(First) => Buf.push(First),
				}

				// Drain everything already queued without blocking.
				Rx.recv_many(&mut Buf, 4096).await;

				// One frame - let stragglers accumulate.
				tokio::time::sleep(std::time::Duration::from_millis(16)).await;

				// Drain again after the frame window.
				Rx.recv_many(&mut Buf, 4096).await;

				if Buf.is_empty() {
					continue;
				}

				// Emit single batch; all items share the same AppHandle.
				let Handle = Buf[0].0.clone();

				let Commands:Vec<Value> = Buf.drain(..).map(|(_, V)| V).collect();

				let Count = Commands.len();

				match Handle.emit("sky://command/register", json!({ "commands": Commands })) {
					Ok(()) => {
						dev_log!("sky-emit", "[SkyEmit] ok channel=sky://command/register batch={}", Count);

						// Summary line at the default-visible `commands` tag
						// so `Trace=short` still surfaces the boot burst as
						// `RegisterCommand batch=N` per 16ms window instead
						// of N hidden per-command lines under
						// `command-register`. One line per batch is the
						// natural granularity - matches the rate of the
						// downstream Sky emit.
						dev_log!("commands", "[RegisterCommand] batch={}", Count);
					},
					Err(E) => {
						dev_log!(
							"sky-emit",
							"[SkyEmit] fail channel=sky://command/register batch={} error={}",
							Count,
							E
						);
					},
				}
			}
		});

		CommandBatchChannel { Sender:Tx }
	})
}

pub async fn RegisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {
	let CommandId = Parameter.get("commandId").and_then(Value::as_str).unwrap_or("");

	dev_log!(
		"command-register",
		"[MountainVinegRPCService] Cocoon registered command: {}",
		CommandId
	);

	if CommandId.is_empty() {
		return;
	}

	let Kind = Parameter.get("kind").and_then(Value::as_str).unwrap_or("command").to_string();

	if let Ok(mut Registry) = Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.Registry
		.CommandRegistry
		.lock()
	{
		Registry.insert(
			CommandId.to_string(),
			CommandHandler::Proxied {
				SideCarIdentifier:"cocoon-main".to_string(),
				CommandIdentifier:CommandId.to_string(),
			},
		);
	}

	let Ch = GetOrInitChannel(Service.ApplicationHandle());

	let _ = Ch.Sender.send((
		Service.ApplicationHandle().clone(),
		json!({ "id": CommandId, "commandId": CommandId, "kind": Kind }),
	));
}
