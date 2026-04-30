#![allow(non_snake_case)]

//! Bidirectional streaming multiplexer for the Vine gRPC bus.
//!
//! Owns one bidirectional h2 stream per sidecar. Inbound notifications
//! fan out to the process-wide broadcast
//! (`Vine::Client::SubscribeNotifications`); inbound responses route to the
//! matching pending-request `oneshot` sender. Inbound reverse-RPC requests and
//! cancellations are TODO for a follow-up phase.
//!
//! This is the P14.1 foundation of Patch 14 - it lands the open(),
//! Notify(), Request(), and ReadPump skeleton so subsequent phases can
//! wire `SendNotification` / `SendRequest` to consult the multiplexer
//! when `LAND_VINE_STREAMING=1` is set.

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

/// Outbound queue capacity per multiplexer. Bounded so a stalled
/// sidecar applies backpressure to the producer side instead of
/// burning unbounded heap.
const SINK_CAPACITY:usize = 1024;

/// One multiplexer per sidecar connection. Holds the outbound sink,
/// the pending-request correlation map, and a shared-state shutdown
/// flag.
pub struct Multiplexer {
	SideCarIdentifier:String,
	Sink:mpsc::Sender<Envelope>,
	Pending:Arc<DashMap<u64, oneshot::Sender<GenericResponse>>>,
	NextRequestIdentifier:AtomicU64,
	Closed:AtomicBool,
}

lazy_static! {
	/// Process-wide registry, one entry per sidecar identifier.
	/// Lookup site for `SendNotification` / `SendRequest` to consult
	/// when `LAND_VINE_STREAMING=1`.
	static ref MULTIPLEXERS:Arc<Mutex<HashMap<String, Arc<Multiplexer>>>> = Arc::new(Mutex::new(HashMap::new()));
}

impl Multiplexer {
	/// Open a bidirectional streaming channel against an existing
	/// `CocoonServiceClient`. Spawns the read pump as a detached
	/// tokio task and registers the multiplexer in the global
	/// registry. Returns once the stream is established.
	pub async fn Open(
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

	/// Look up the multiplexer for a sidecar. Returns `None` if no
	/// streaming connection has been opened for that sidecar (the
	/// caller should fall back to the unary path).
	pub fn Lookup(SideCarIdentifier:&str) -> Option<Arc<Self>> { MULTIPLEXERS.lock().get(SideCarIdentifier).cloned() }

	/// Drop the registry entry. Called by the read-pump when the
	/// stream closes.
	pub fn Deregister(SideCarIdentifier:&str) { MULTIPLEXERS.lock().remove(SideCarIdentifier); }

	/// Send a notification frame (fire-and-forget). Non-blocking
	/// modulo Sink backpressure (capacity `SINK_CAPACITY`).
	pub async fn Notify(&self, Method:String, Parameters:Value) -> Result<(), VineError> {
		if self.Closed.load(Ordering::Relaxed) {
			return Err(VineError::ClientNotConnected(self.SideCarIdentifier.clone()));
		}
		let Bytes = serde_json::to_vec(&Parameters)?;
		let Frame = Envelope {
			payload:Some(Payload::Notification(GenericNotification { method:Method, parameter:Bytes })),
		};
		self.Sink
			.send(Frame)
			.await
			.map_err(|_| VineError::RPCError(format!("Sink closed for sidecar {}", self.SideCarIdentifier)))
	}

	/// Send a request and await the matching response. Cancels the
	/// pending entry on timeout. The future is `Send + 'static`-clean
	/// so callers can drive it inside `tokio::select!` for finer-
	/// grained cancellation.
	pub async fn Request(&self, Method:String, Parameters:Value, Timeout:Duration) -> Result<Value, VineError> {
		if self.Closed.load(Ordering::Relaxed) {
			return Err(VineError::ClientNotConnected(self.SideCarIdentifier.clone()));
		}
		let Identifier = self.NextRequestIdentifier.fetch_add(1, Ordering::Relaxed);
		let (Tx, Rx) = oneshot::channel();
		self.Pending.insert(Identifier, Tx);

		let Bytes = serde_json::to_vec(&Parameters)?;
		let MethodForError = Method.clone();
		let Frame = Envelope {
			payload:Some(Payload::Request(GenericRequest {
				request_identifier:Identifier,
				method:Method,
				parameter:Bytes,
			})),
		};

		if self.Sink.send(Frame).await.is_err() {
			self.Pending.remove(&Identifier);
			return Err(VineError::RPCError(format!(
				"Sink closed for sidecar {}",
				self.SideCarIdentifier
			)));
		}

		match tokio::time::timeout(Timeout, Rx).await {
			Ok(Ok(Response)) => {
				if let Some(Error) = Response.error {
					return Err(VineError::RPCError(format!("code={} message={}", Error.code, Error.message)));
				}
				if Response.result.is_empty() {
					return Ok(Value::Null);
				}
				serde_json::from_slice::<Value>(&Response.result).map_err(|E| VineError::SerializationError(E))
			},
			Ok(Err(_)) => {
				self.Pending.remove(&Identifier);
				Err(VineError::RPCError(
					"response sender closed (peer disconnect mid-request)".into(),
				))
			},
			Err(_) => {
				self.Pending.remove(&Identifier);
				Err(VineError::RequestTimeout {
					SideCarIdentifier:self.SideCarIdentifier.clone(),
					MethodName:MethodForError,
					TimeoutMilliseconds:Timeout.as_millis() as u64,
				})
			},
		}
	}

	/// Send a Cancel frame asking the peer to abort an in-flight
	/// request matching `RequestIdentifier`. Best-effort; the peer
	/// chooses whether to honour it.
	pub async fn Cancel(&self, RequestIdentifier:u64) -> Result<(), VineError> {
		if self.Closed.load(Ordering::Relaxed) {
			return Ok(());
		}
		let Frame = Envelope {
			payload:Some(Payload::Cancel(CancelOperationRequest { request_identifier_to_cancel:RequestIdentifier })),
		};
		let _ = self.Sink.send(Frame).await;
		Ok(())
	}

	pub fn IsClosed(&self) -> bool { self.Closed.load(Ordering::Relaxed) }

	pub fn SideCarIdentifierBorrow(&self) -> &str { &self.SideCarIdentifier }
}

/// Drain the inbound side of the bidirectional stream. Notifications
/// fan out to the process-wide broadcast; responses wake the parked
/// `Request` future. Reverse-RPC requests and cancellations are
/// recorded for a follow-up phase.
async fn ReadPump(mut Stream:Streaming<Envelope>, State:Arc<Multiplexer>) {
	use futures_util::StreamExt;

	while let Some(FrameResult) = Stream.next().await {
		let Frame = match FrameResult {
			Ok(F) => F,
			Err(Status) => {
				dev_log!("grpc", "[Vine::Multiplexer] read err on {}: {}", State.SideCarIdentifier, Status);
				break;
			},
		};
		let Payload = match Frame.payload {
			Some(P) => P,
			None => continue,
		};

		match Payload {
			Payload::Notification(N) => {
				let Parameters:Value = if N.parameter.is_empty() {
					Value::Null
				} else {
					serde_json::from_slice(&N.parameter).unwrap_or(Value::Null)
				};
				super::Client::PublishNotificationFromMux(&State.SideCarIdentifier, &N.method, &Parameters);
			},
			Payload::Response(R) => {
				let Identifier = R.request_identifier;
				if let Some((_, Sender)) = State.Pending.remove(&Identifier) {
					let _ = Sender.send(R);
				}
				// A Response with no matching pending entry is a
				// duplicate or post-cancel arrival; drop silently.
			},
			Payload::Request(_) => {
				// TODO P14.1.1: dispatch the inbound (reverse-RPC)
				// request to the same handler tree the unary path
				// uses, then enqueue the GenericResponse onto Sink.
				// For now we drop, which is safe: the unary path is
				// still authoritative until phase P14.4 lands the
				// streaming handler tree on Cocoon side.
			},
			Payload::Cancel(_) => {
				// TODO P14.1.2: signal abort for the in-flight
				// handler. For now no-op (the unary path doesn't
				// support cancel either).
			},
		}
	}

	State.Closed.store(true, Ordering::Relaxed);

	// Drain pending senders with disconnect errors so awaiting
	// fibers don't hang forever.
	let Keys:Vec<u64> = State.Pending.iter().map(|R| *R.key()).collect();
	for Key in Keys {
		if let Some((_, Sender)) = State.Pending.remove(&Key) {
			let _ = Sender.send(GenericResponse {
				request_identifier:Key,
				result:Vec::new(),
				error:Some(RpcError { code:-32099, message:"stream closed".into(), data:Vec::new() }),
			});
		}
	}

	Multiplexer::Deregister(&State.SideCarIdentifier);
	dev_log!("grpc", "[Vine::Multiplexer] closed sidecar={}", State.SideCarIdentifier);
}
