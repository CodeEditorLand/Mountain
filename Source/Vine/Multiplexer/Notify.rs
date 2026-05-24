//! `Multiplexer::Notify`

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

pub fn Fn(This:&Struct, Method:String, Parameters:Value) -> Result<(), VineError> {
	if This.Closed.load(Ordering::Relaxed) {
		return Err(VineError::ClientNotConnected(This.SideCarIdentifier.clone()));
	}

	let Bytes = serde_json::to_vec(&Parameters)?;

	let Frame = Envelope {
		payload:Some(Payload::Notification(GenericNotification { method:Method, parameter:Bytes })),
	};

	This.Sink
		.Send(Frame)
		.await
		.map_err(|_| VineError::RPCError(format!("Sink closed for sidecar {}", This.SideCarIdentifier)))
}
