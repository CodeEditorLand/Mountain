//! `Multiplexer::Request`

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicBool, AtomicU64, Ordering},
	},
	time::Duration,
};

use dashmap::DashMap;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;

use super::{
	Error::VineError,
	Generated::{
		CancelOperationRequest,
		Envelope,
		GenericNotification,
		GenericRequest,
		GenericResponse,
		RpcError,
		cocoon_service_client::CocoonServiceClient,
		envelope::Payload,
	},
	Struct,
};
use crate::dev_log;

pub fn Fn(This:&Struct, Method:String, Parameters:Value, Timeout:Duration) -> Result<Value, VineError> {
	if This.Closed.load(Ordering::Relaxed) {
		return Err(VineError::ClientNotConnected(This.SideCarIdentifier.clone()));
	}

	let Identifier = This.NextRequestIdentifier.fetch_add(1, Ordering::Relaxed);

	let (Tx, Rx) = oneshot::channel();

	This.Pending.insert(Identifier, Tx);

	let Bytes = serde_json::to_vec(&Parameters)?;

	let MethodForError = Method.clone();

	let Frame = Envelope {
		payload:Some(Payload::Request(GenericRequest {
			request_identifier:Identifier,
			method:Method,
			parameter:Bytes,
		})),
	};

	if This.Sink.send(Frame).await.is_err() {
		This.Pending.remove(&Identifier);

		return Err(VineError::RPCError(format!(
			"Sink closed for sidecar {}",
			This.SideCarIdentifier
		)));
	}

	match tokio::time::timeout(Timeout, Rx).await {
		Ok(Ok(Response)) => {
			if let Some(Error) = Response.error {
				return Err(VineError::RPCError(format!("code={} message={}", Error.code, Error.Message)));
			}

			if Response.result.is_empty() {
				return Ok(Value::Null);
			}

			serde_json::from_slice::<Value>(&Response.result).map_err(|E| VineError::SerializationError(E))
		},

		Ok(Err(_)) => {
			This.Pending.remove(&Identifier);

			Err(VineError::RPCError(
				"response sender closed (peer disconnect mid-request)".into(),
			))
		},

		Err(_) => {
			This.Pending.remove(&Identifier);

			Err(VineError::RequestTimeout {
				SideCarIdentifier:This.SideCarIdentifier.clone(),
				MethodName:MethodForError,
				TimeoutMilliseconds:Timeout.as_millis() as u64,
			})
		},
	}
}
