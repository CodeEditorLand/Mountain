#![allow(non_snake_case)]

//! Fire-and-forget notification to a sidecar. No response, no per-call
//! timeout. Prefers the streaming multiplexer under
//! `LAND_VINE_STREAMING=1`; falls through to unary on any failure.
//! After a successful wire send, fans out via `PublishNotification::Fn`
//! so broadcast subscribers (Effect-TS fibers, OTel emitters, future
//! Mist-WS bridge, dev log) can observe the same flow concurrently.

use serde_json::{Value, to_vec};

use crate::{
	Vine::{
		Client::{
			IsShuttingDown,
			PublishNotification,
			Shared::{RecordSideCarFailure, SIDECAR_CLIENTS, UpdateSideCarActivity, ValidateMessageSize},
		},
		Error::VineError,
		Generated::GenericNotification,
	},
	dev_log,
};

pub async fn Fn(SideCarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {
	if IsShuttingDown::Fn() {
		return Ok(());
	}
	if Method.is_empty() || Method.len() > 128 {
		return Err(VineError::RPCError(
			"Method name must be between 1 and 128 characters".to_string(),
		));
	}

	if std::env::var("LAND_VINE_STREAMING").as_deref() == Ok("1") {
		if let Some(Mux) = crate::Vine::Multiplexer::Multiplexer::Lookup(&SideCarIdentifier) {
			if !Mux.IsClosed() {
				let MethodForPublish = Method.clone();
				let ParametersForPublish = Parameters.clone();
				match Mux.Notify(Method.clone(), Parameters.clone()).await {
					Ok(()) => {
						UpdateSideCarActivity(&SideCarIdentifier);
						PublishNotification::Fn(&SideCarIdentifier, &MethodForPublish, &ParametersForPublish);
						return Ok(());
					},
					Err(Error) => {
						dev_log!(
							"grpc",
							"warn: [VineClient::SendNotification] streaming send failed for '{}' ({}); falling back \
							 to unary",
							SideCarIdentifier,
							Error
						);
					},
				}
			}
		}
	}

	let ParameterBytes = to_vec(&Parameters)?;
	ValidateMessageSize(&ParameterBytes)?;

	let mut Client = {
		let Pool = SIDECAR_CLIENTS.lock();
		Pool.get(&SideCarIdentifier).cloned()
	};

	if let Some(ref mut Client) = Client {
		let MethodForPublish = Method.clone();
		let Request = GenericNotification { method:Method, parameter:ParameterBytes };

		match Client.send_mountain_notification(Request).await {
			Ok(_) => {
				UpdateSideCarActivity(&SideCarIdentifier);
				dev_log!(
					"grpc",
					"[VineClient] Notification sent successfully to sidecar '{}'",
					SideCarIdentifier
				);
				PublishNotification::Fn(&SideCarIdentifier, &MethodForPublish, &Parameters);
				Ok(())
			},
			Err(Status) => {
				RecordSideCarFailure(&SideCarIdentifier);
				dev_log!(
					"grpc",
					"error: [VineClient] Failed to send notification to sidecar '{}': {}",
					SideCarIdentifier,
					Status
				);
				Err(VineError::from(Status))
			},
		}
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier))
	}
}
