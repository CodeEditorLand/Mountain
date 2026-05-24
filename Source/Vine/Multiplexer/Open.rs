//! `Multiplexer::Open`

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

pub fn Fn(
	SideCarIdentifier:String,

	mut Client:CocoonServiceClient<tonic::transport::Channel>,
) -> Result<Arc<Self>, VineError> {
	let (Sink, OutboundReceiver) = mpsc::channel::<Envelope>(SINK_CAPACITY);

	let OutboundStream = ReceiverStream::new(OutboundReceiver);

	let Response = Client
		.open_channel_from_mountain(OutboundStream)
		.await
		.map_err(|S| VineError::RPCError(format!("OpenChannelFromMountain failed: {}", S)))?;

	let InboundStream:Streaming<Envelope> = Response.into_inner();

	let SelfReference = Arc::new(Self {
		SideCarIdentifier:SideCarIdentifier.clone(),
		Sink,
		Pending:Arc::new(DashMap::new()),
		NextRequestIdentifier:AtomicU64::new(1),
		Closed:AtomicBool::new(false),
	});

	// Spawn the read pump.
	let SelfForReadPump = SelfReference.clone();

	tokio::spawn(async move {
		ReadPump(InboundStream, SelfForReadPump).await;
	});

	// Register globally so consumers can look us up.
	MULTIPLEXERS.lock().insert(SideCarIdentifier, SelfReference.clone());

	Ok(SelfReference)
}
