//! Mountain-side compat surface for the per-channel coalescing buffer.
//! The canonical implementation lives in
//! `::Vine::Server::Notification::OutputChannelCoalesce`. This thin
//! delegator preserves the Mountain-path `TryEnqueue` for any historical
//! caller; the Vine-side handler is the steady-state path.

use std::sync::Arc;

use tauri::AppHandle;

use crate::Vine::Server::VineHostImpl::TauriRendererEmitter;

/// Trys enqueue.
pub fn TryEnqueue(Handle:&AppHandle, Channel:String, Value:String) -> bool {
	let Emitter = Arc::new(TauriRendererEmitter::New(Handle.clone()));

	::Vine::Server::Notification::OutputChannelCoalesce::TryEnqueue(Emitter, Channel, Value)
}
