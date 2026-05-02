//! # VineSubscribeCommand
//!
//! Tauri command surface that exposes the process-wide Vine
//! notification broadcast (`Vine::Client::SubscribeNotifications`)
//! to Sky / Wind via a Tauri Channel<NotificationFramePayload>.
//!
//! Wind / Sky subscribers consume each frame as it arrives - same
//! ordering, same drop-oldest semantics as the in-process Rust
//! subscribers. The Effect-TS Layer in
//! `Element/Wind/Source/Effect/Vine/NotificationStream.ts` wraps this
//! into a `Stream<NotificationFrame>`.
//!
//! Frame shape on the wire (serde_json):
//!
//! ```json
//! {
//!   "sideCarIdentifier": "cocoon-main",
//!   "method": "Diagnostic.Set",
//!   "parameters": <payload>,
//!   "timestampNanos": 17775062973342540
//! }
//! ```

use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;

use crate::{Vine::Client::SubscribeNotifications::Fn as SubscribeNotifications, dev_log};

/// Webview-facing notification frame. Mirror of the Rust
/// `Vine::Client::NotificationFrame` with camelCase field names per
/// Land's wire convention. Field renames keep Sky's TS bindings
/// stable even if the Rust struct evolves.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFramePayload {
	pub side_car_identifier:String,
	pub method:String,
	pub parameters:Value,
	pub timestamp_nanos:u64,
}

/// Subscribe to the Vine notification broadcast. Each call returns a
/// fresh subscription (own broadcast::Receiver) and spawns a tokio
/// task that drains the receiver onto the supplied Tauri Channel
/// until the webview drops it. Drop-oldest at capacity 4096; slow
/// subscribers may see gaps but never block the producer.
///
/// Returns the current subscriber count (post-subscribe) so the
/// frontend can verify the channel is registered.
#[tauri::command]
pub async fn vine_subscribe_notifications(channel:Channel<NotificationFramePayload>) -> Result<usize, String> {
	let mut Receiver = SubscribeNotifications();
	let SubscriberCount = crate::Vine::Client::SubscriberCount::Fn();
	dev_log!(
		"grpc",
		"[VineSubscribe] webview subscribed; total_subscribers={}",
		SubscriberCount
	);

	tokio::spawn(async move {
		loop {
			match Receiver.recv().await {
				Ok(Frame) => {
					let Payload = NotificationFramePayload {
						side_car_identifier:Frame.SideCarIdentifier,
						method:Frame.Method,
						parameters:Frame.Parameters,
						timestamp_nanos:Frame.TimestampNanos,
					};
					if let Err(Error) = channel.send(Payload) {
						// Channel closed - the webview disposed its
						// subscription. Exit the drain task.
						dev_log!("grpc", "[VineSubscribe] channel closed ({}); ending drain task", Error);
						break;
					}
				},
				Err(tokio::sync::broadcast::error::RecvError::Lagged(N)) => {
					// Subscriber fell behind; drop-oldest semantics.
					// Surface the gap count so the consumer can decide
					// whether to refresh state.
					dev_log!("grpc", "warn: [VineSubscribe] subscriber lagged; dropped {} frames", N);
				},
				Err(tokio::sync::broadcast::error::RecvError::Closed) => {
					// Producer side closed (process shutdown).
					break;
				},
			}
		}
	});

	Ok(SubscriberCount)
}

/// Diagnostic: how many active subscribers exist on the broadcast.
/// Useful from the frontend for verifying that prior subscriptions
/// haven't leaked across reloads.
#[tauri::command]
pub async fn vine_subscriber_count() -> Result<usize, String> { Ok(crate::Vine::Client::SubscriberCount::Fn()) }
