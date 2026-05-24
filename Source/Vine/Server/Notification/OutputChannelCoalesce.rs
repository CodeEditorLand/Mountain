//! Per-channel coalescing buffer for `outputChannel.append` notifications.
//!
//! Cocoon's Git extension emits 30+ `append` notifications per `git status`
//! (one per `[trace] [OperationManager][...]` line, one per executed
//! sub-command). Each one previously:
//!
//!   1. Crossed the Cocoon → Mountain gRPC notification boundary.
//!   2. Fired its own `Tauri::Emitter::emit("sky://output/append")` round-trip.
//!   3. Wrote its own dev_log entry.
//!
//! For a workspace with the Git extension actively probing repo state on
//! file changes, the volume of `[OperationManager]` traces alone accounted
//! for ~1.9k lines of one 28k-line session log.
//!
//! This atom buffers appends per-channel for a short window
//! (`COALESCE_WINDOW`) and flushes the concatenated payload as a single
//! Sky emit + a single dev_log line. The downstream Output panel still
//! sees identical text - just delivered in larger chunks - which matches
//! the user-perceived UX of an output channel (it scrolls in chunks, not
//! character-by-character).
//!
//! ## Why this is safe
//!
//! - Per-channel buffer means ordering is preserved within a channel.
//! - Append-only semantics mean partial-payload visibility cannot expose torn
//!   writes - the buffered text is always a prefix of the eventual full
//!   payload.
//! - `Tauri::Emitter` serialises emits per channel; the flush task running on
//!   the tokio runtime keeps the same back-pressure shape the per-call path
//!   had.
//!
//! ## Disable hook
//!
//! `OutputCoalesce=0` reverts to per-append emit (debugging
//! synchronisation issues where a single append must be flushed
//! immediately to disk).

use std::{
	collections::HashMap,
	sync::{Mutex as StandardMutex, OnceLock},
	time::Duration,
};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::dev_log;

/// Maximum delay between an append arriving and its flush to Sky. Tuned
/// against the FSEvents / Git-extension 16ms tick - one frame is enough
/// for a `git status` burst to fully accumulate without introducing a
/// human-perceptible scroll lag.
const COALESCE_WINDOW:Duration = Duration::from_millis(50);

/// Maximum buffered bytes per channel before a forced flush. Caps memory
/// for any channel emitting unbounded text (a build extension piping
/// `cargo build` stdout) before the timer fires.
const MAX_BUFFERED_BYTES:usize = 64 * 1024;

struct PendingAppend {
	Channel:String,

	Value:String,
}

struct CoalesceChannel {
	Sender:UnboundedSender<(AppHandle, PendingAppend)>,
}

static COALESCE_CHANNEL:OnceLock<CoalesceChannel> = OnceLock::new();

fn IsDisabled() -> bool { matches!(std::env::var("OutputCoalesce").as_deref(), Ok("0") | Ok("false")) }

fn GetOrInitChannel() -> &'static CoalesceChannel {
	COALESCE_CHANNEL.get_or_init(|| {
		let (Tx, mut Rx) = unbounded_channel::<(AppHandle, PendingAppend)>();

		tokio::spawn(async move {
			// Per-channel pending buffer. Two-level map: channel name →
			// accumulated text. AppHandle is shared across channels (one
			// per process) so we stash it alongside.
			let Buffers:StandardMutex<HashMap<String, String>> = StandardMutex::new(HashMap::new());

			loop {
				let Received = Rx.recv().await;

				let (Handle, First) = match Received {
					None => break,
					Some(Pair) => Pair,
				};

				// Append to per-channel buffer.
				{
					let mut Guard = match Buffers.lock() {
						Ok(G) => G,
						Err(_) => continue,
					};

					let Slot = Guard.entry(First.Channel.clone()).or_default();

					Slot.push_str(&First.Value);

					if Slot.len() >= MAX_BUFFERED_BYTES {
						let Payload = std::mem::take(Slot);

						drop(Guard);

						FlushOne(&Handle, &First.Channel, &Payload);

						continue;
					}
				}

				// Drain everything already queued without blocking.
				let mut Drain:Vec<(AppHandle, PendingAppend)> = Vec::new();

				let Drained = Rx.recv_many(&mut Drain, 4096).await;

				let _ = Drained;

				for (_, Pending) in Drain.drain(..) {
					if let Ok(mut Guard) = Buffers.lock() {
						let Slot = Guard.entry(Pending.Channel).or_default();
						Slot.push_str(&Pending.Value);
					}
				}

				// Window pause - let stragglers accumulate inside the
				// 50 ms frame. Anything received during the sleep is
				// drained again before flush.
				tokio::time::sleep(COALESCE_WINDOW).await;

				let mut LateDrain:Vec<(AppHandle, PendingAppend)> = Vec::new();

				let _ = Rx.recv_many(&mut LateDrain, 4096).await;

				for (_, Pending) in LateDrain.drain(..) {
					if let Ok(mut Guard) = Buffers.lock() {
						let Slot = Guard.entry(Pending.Channel).or_default();
						Slot.push_str(&Pending.Value);
					}
				}

				// Flush every non-empty channel.
				let HandleForFlush = Handle.clone();

				let Snapshots = {
					match Buffers.lock() {
						Ok(mut Guard) => {
							Guard
								.iter_mut()
								.filter(|(_, V)| !V.is_empty())
								.map(|(K, V)| (K.clone(), std::mem::take(V)))
								.collect::<Vec<_>>()
						},
						Err(_) => continue,
					}
				};

				for (Channel, Payload) in Snapshots {
					FlushOne(&HandleForFlush, &Channel, &Payload);
				}
			}
		});

		CoalesceChannel { Sender:Tx }
	})
}

fn FlushOne(Handle:&AppHandle, Channel:&str, Payload:&str) {
	let _ = Handle.emit(
		"sky://output/append",
		json!({
			"channel": Channel,
			"value": Payload,
		}),
	);

	// One log line per flush instead of one per append. The git
	// channel keeps its `grpc`-tag visibility for SCM activation
	// diagnosis; other channels stay under `output-verbose`.
	let IsGitFamily = Channel.eq_ignore_ascii_case("git")
		|| Channel.eq_ignore_ascii_case("source control")
		|| Channel.eq_ignore_ascii_case("scm");

	let LineCount = Payload.matches('\n').count();

	if IsGitFamily {
		dev_log!(
			"grpc",
			"[OutputChannel:{}] flush bytes={} lines~{}",
			Channel,
			Payload.len(),
			LineCount
		);
	} else {
		dev_log!(
			"output-verbose",
			"[OutputChannel] flush channel={} bytes={} lines~{}",
			Channel,
			Payload.len(),
			LineCount
		);
	}
}

/// Submit a pending append for coalescing. Returns `true` when the
/// item was enqueued (the coalescer will flush within `COALESCE_WINDOW`),
/// `false` when coalescing is disabled and the caller must flush
/// inline.
pub fn Fn(Handle:&AppHandle, Channel:String, Value:String) -> bool {
	if IsDisabled() {
		return false;
	}

	let Ch = GetOrInitChannel();

	let _ = Ch.Sender.send((Handle.clone(), PendingAppend { Channel, Value }));

	true
}
