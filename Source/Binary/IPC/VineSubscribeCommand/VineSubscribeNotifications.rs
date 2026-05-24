//! `VineSubscribeCommand::VineSubscribeNotifications`

use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use crate::{Vine::Client::SubscribeNotifications::Fn as SubscribeNotifications, dev_log};

/// Subscribe to the Vine notification broadcast. Each call returns a
/// fresh subscription (own broadcast::Receiver) and spawns a tokio
/// task that drains the receiver onto the supplied Tauri Channel
/// until the webview drops it. Drop-oldest at capacity 4096; slow
/// subscribers may see gaps but never block the producer.
///
/// Returns the current subscriber count (post-subscribe) so the
/// frontend can verify the channel is registered.
#[tauri::command]
pub async fn Fn(channel:Channel<NotificationFramePayload>) -> Result<usize, String> {
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
