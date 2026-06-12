//! Detached read pump for the Mountain↔Cocoon bidirectional `Envelope`
//! stream (`open_channel_from_mountain`). Routes each inbound frame:
//! Notification → unary `GenericNotification::Dispatcher`; Request →
//! unary `GenericRequest::Dispatcher` with the response pushed back via
//! the outbound sender; Response → unexpected on this direction, logged
//! and dropped; Cancel → deliberate no-op (the unary path has no cancel
//! support). Removes the `CHANNEL_REGISTRY` entry when the stream closes
//! so stale senders are not retained.

use tokio::sync::mpsc;

use crate::dev_log;

pub(crate) fn Fn(
	Service:super::CocoonServiceImpl,

	ChannelId:u64,

	mut Inbound:tonic::Streaming<::Vine::Generated::Envelope>,

	OutTx:mpsc::Sender<::Vine::Generated::Envelope>,
) {
	use ::Vine::Generated::{Envelope, GenericResponse, RpcError, envelope::Payload};
	use futures_util::StreamExt;

	tauri::async_runtime::spawn(async move {
		while let Some(FrameResult) = Inbound.next().await {
			let Frame = match FrameResult {
				Ok(F) => F,

				Err(Status) => {
					dev_log!("cocoon", "[CocoonService] channel_id={} inbound error: {}", ChannelId, Status);

					break;
				},
			};

			let Payload = match Frame.payload {
				Some(P) => P,

				None => continue,
			};

			match Payload {
				Payload::Notification(N) => {
					// Reuse the unary notification dispatcher verbatim.
					let _ = super::GenericNotification::Dispatcher::Fn(&Service, tonic::Request::new(N)).await;
				},

				Payload::Request(R) => {
					let RequestId = R.request_identifier;

					// Reuse the unary request dispatcher; wrap the result
					// into a Response envelope and push it back.
					let Wrapped = tonic::Request::new(R);

					let Response = match super::GenericRequest::Dispatcher::Fn(&Service, Wrapped).await {
						Ok(GrpcResponse) => {
							let Inner = GrpcResponse.into_inner();

							Envelope { payload:Some(Payload::Response(Inner)) }
						},

						Err(Status) => {
							Envelope {
								payload:Some(Payload::Response(GenericResponse {
									request_identifier:RequestId,
									result:Vec::new(),
									error:Some(RpcError {
										code:Status.code() as i32,
										message:Status.message().to_string(),
										data:Vec::new(),
									}),
								})),
							}
						},
					};

					if OutTx.send(Response).await.is_err() {
						// Receiver closed - peer disconnected.
						break;
					}
				},

				Payload::Response(_) => {
					// Responses on the Mountain-inbound direction are
					// unexpected; drop silently.
					dev_log!(
						"cocoon",
						"[CocoonService] channel_id={} unexpected Response frame; ignored",
						ChannelId
					);
				},

				Payload::Cancel(_) => {
					// Best-effort cancel; the unary path has no cancel
					// support so this is a deliberate no-op.
				},
			}
		}

		// Pump exited - remove the registry entry so stale senders are
		// not retained.
		super::CHANNEL_REGISTRY.remove(&ChannelId);

		dev_log!(
			"cocoon",
			"[CocoonService] open_channel_from_mountain channel_id={} closed",
			ChannelId
		);
	});
}
