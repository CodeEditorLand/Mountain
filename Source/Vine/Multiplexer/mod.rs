pub mod Open;
pub mod Lookup;
pub mod Deregister;
pub mod Notify;
pub mod Request;
pub mod Cancel;
pub mod IsClosed;
pub mod SideCarIdentifierBorrow;

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
};
use crate::dev_log;

/// One multiplexer per sidecar connection. Holds the outbound sink,
/// the pending-request correlation map, and a shared-state shutdown
/// flag.
pub struct Struct {
	SideCarIdentifier:String,

	Sink:mpsc::Sender<Envelope>,

	Pending:Arc<DashMap<u64, oneshot::Sender<GenericResponse>>>,

	NextRequestIdentifier:AtomicU64,

	Closed:AtomicBool,
}
