//! Send a request and await a response. Validates method-name length
//! and message size, prefers the streaming multiplexer when
//! `LAND_VINE_STREAMING=1` is on (falls through to unary on any failure
//! except the authoritative streaming-path timeout), enforces a per-call
//! timeout via `tokio::time::timeout`, and updates per-connection
//! activity / failure metadata on completion.

use std::time::Duration;

use serde_json::{Value, from_slice, to_vec};
use tokio::time::timeout;

use crate::{
	Vine::{
		Client::{
			IsShuttingDown,
			Shared::{
				DEFAULT_TIMEOUT_MS,
				RecordSideCarFailure,
				SIDECAR_CLIENTS,
				UpdateSideCarActivity,
				ValidateMessageSize,
			},
		},
		Error::VineError,
		Generated::GenericRequest,
	},
	dev_log,
};

pub async fn Fn(
	SideCarIdentifier:&str,

	Method:String,

	Parameters:Value,

	TimeoutMilliseconds:u64,
) -> Result<Value, VineError> {
	if IsShuttingDown::Fn() {
		return Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()));
	}

	if Method.is_empty() || Method.len() > 128 {
		return Err(VineError::RPCError(
			"Method name must be between 1 and 128 characters".to_string(),
		));
	}

	let TimeoutDuration =
		Duration::from_millis(if TimeoutMilliseconds > 0 { TimeoutMilliseconds } else { DEFAULT_TIMEOUT_MS });

	if std::env::var("LAND_VINE_STREAMING").as_deref() == Ok("1") {
		if let Some(Mux) = crate::Vine::Multiplexer::Multiplexer::Lookup(SideCarIdentifier) {
			if !Mux.IsClosed() {
				match Mux.Request(Method.clone(), Parameters.clone(), TimeoutDuration).await {
					Ok(Result_) => {
						UpdateSideCarActivity(SideCarIdentifier);

						return Ok(Result_);
					},

					Err(VineError::RequestTimeout { .. }) => {
						return Err(VineError::RequestTimeout {
							SideCarIdentifier:SideCarIdentifier.to_string(),
							MethodName:Method,
							TimeoutMilliseconds:TimeoutDuration.as_millis() as u64,
						});
					},

					Err(Error) => {
						dev_log!(
							"grpc",
							"warn: [VineClient::SendRequest] streaming send failed for '{}::{}' ({}); falling back to \
							 unary",
							SideCarIdentifier,
							Method,
							Error
						);
					},
				}
			}
		}
	}

	let ParameterBytes =
		to_vec(&Parameters).map_err(|E| VineError::RPCError(format!("Failed to serialize parameters: {}", E)))?;

	ValidateMessageSize(&ParameterBytes)?;

	let Client = {
		let Pool = SIDECAR_CLIENTS.lock();

		Pool.get(SideCarIdentifier).cloned()
	};

	let Some(mut Client) = Client else {
		return Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()));
	};

	use std::sync::atomic::{AtomicU64, Ordering as AO};

	static REQ_SEQ:AtomicU64 = AtomicU64::new(1);

	let RequestIdentifier = REQ_SEQ.fetch_add(1, AO::Relaxed);

	let MethodForLog = Method.clone();

	let Request = GenericRequest { request_identifier:RequestIdentifier, method:Method, parameter:ParameterBytes };

	let Result_ = timeout(TimeoutDuration, Client.process_mountain_request(Request)).await;

	match Result_ {
		Ok(Ok(Response)) => {
			UpdateSideCarActivity(SideCarIdentifier);

			dev_log!(
				"grpc",
				"[VineClient] Request sent successfully to sidecar '{}': method='{}'",
				SideCarIdentifier,
				MethodForLog
			);

			let InnerResponse = Response.into_inner();

			let ResultBytes = InnerResponse.result;

			let ResultValue:Value = from_slice(&ResultBytes)
				.map_err(|E| VineError::RPCError(format!("Failed to deserialize response: {}", E)))?;

			if let Some(ErrorData) = InnerResponse.error {
				return Err(VineError::RPCError(format!(
					"RPC error from sidecar: code={}, message={}",
					ErrorData.code, ErrorData.Message
				)));
			}

			Ok(ResultValue)
		},

		Ok(Err(Status)) => {
			RecordSideCarFailure(SideCarIdentifier);

			Err(VineError::RPCError(format!("gRPC error: {}", Status)))
		},

		Err(_) => {
			RecordSideCarFailure(SideCarIdentifier);

			Err(VineError::RequestTimeout {
				SideCarIdentifier:SideCarIdentifier.to_string(),
				MethodName:MethodForLog,
				TimeoutMilliseconds:TimeoutDuration.as_millis() as u64,
			})
		},
	}
}
