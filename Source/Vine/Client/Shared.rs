#![allow(non_snake_case)]

//! Module-private state for the Vine client: connection pool, per-
//! connection metadata, the broadcast fan-out, the shutdown flag, plus
//! the constants and message-size validator that every entry-point shares.

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Instant,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use crate::Vine::{Client::NotificationFrame, Error::VineError, Generated::cocoon_service_client::CocoonServiceClient};

/// Cocoon gRPC client over a tonic transport channel.
pub type CocoonClient = CocoonServiceClient<tonic::transport::Channel>;

/// Default timeout for RPC calls.
pub const DEFAULT_TIMEOUT_MS:u64 = 5000;
/// Maximum number of retry attempts for failed connections.
pub const MAX_RETRY_ATTEMPTS:usize = 3;
/// Base delay between retry attempts.
pub const RETRY_BASE_DELAY_MS:u64 = 100;
/// Maximum message size for validation (4 MB to match the tonic default).
pub const MAX_MESSAGE_SIZE_BYTES:usize = 4 * 1024 * 1024;
/// Health-check interval.
pub const HEALTH_CHECK_INTERVAL_MS:u64 = 30000;
/// Connection timeout (currently unused - kept for the streaming variant).
#[allow(dead_code)]
pub const CONNECTION_TIMEOUT_MS:u64 = 10000;

/// Notification broadcast capacity (drop-oldest when full). 4096 covers
/// the worst-case storms (sky://diagnostics/changed at 50-200/s during
/// rust-analyzer cargo-check) with margin.
pub const NOTIFICATION_BROADCAST_CAPACITY:usize = 4096;

/// Connection metadata tracking health and last activity.
pub struct ConnectionMetadata {
	pub LastActivity:Instant,
	pub FailureCount:usize,
	pub IsHealthy:bool,
}

lazy_static! {
	pub static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> = Arc::new(Mutex::new(HashMap::new()));
	pub static ref CONNECTION_METADATA: Arc<Mutex<HashMap<String, ConnectionMetadata>>> =
		Arc::new(Mutex::new(HashMap::new()));
	pub static ref NOTIFICATION_BROADCAST: tokio::sync::broadcast::Sender<NotificationFrame::Struct> = {
		let (Sender, _) = tokio::sync::broadcast::channel(NOTIFICATION_BROADCAST_CAPACITY);
		Sender
	};
}

/// Process-wide shutdown flag. Set to `true` once Mountain has issued
/// `$shutdown` (or SIGKILL'd) Cocoon. After that point all
/// `SendNotification` / `SendRequest` calls short-circuit.
pub static SHUTDOWN_FLAG:AtomicBool = AtomicBool::new(false);

pub fn ShutdownFlagStore(Value:bool) { SHUTDOWN_FLAG.store(Value, Ordering::Relaxed); }
pub fn ShutdownFlagLoad() -> bool { SHUTDOWN_FLAG.load(Ordering::Relaxed) }

/// Increment the failure counter and mark the connection unhealthy.
pub fn RecordSideCarFailure(SideCarIdentifier:&str) {
	let mut Metadata = CONNECTION_METADATA.lock();
	if let Some(Connection) = Metadata.get_mut(SideCarIdentifier) {
		Connection.FailureCount += 1;
		Connection.IsHealthy = false;
	}
}

/// Refresh the last-activity timestamp and reset the failure counter.
pub fn UpdateSideCarActivity(SideCarIdentifier:&str) {
	let mut Metadata = CONNECTION_METADATA.lock();
	if let Some(Connection) = Metadata.get_mut(SideCarIdentifier) {
		Connection.LastActivity = Instant::now();
		Connection.FailureCount = 0;
		Connection.IsHealthy = true;
	}
}

/// Reject messages above `MAX_MESSAGE_SIZE_BYTES` to bound the worst-case
/// gRPC frame. Mirrors tonic's own check so we don't pay the codec round-
/// trip for an oversize payload.
pub fn ValidateMessageSize(Data:&[u8]) -> Result<(), VineError> {
	if Data.len() > MAX_MESSAGE_SIZE_BYTES {
		Err(VineError::MessageTooLarge { ActualSize:Data.len(), MaxSize:MAX_MESSAGE_SIZE_BYTES })
	} else {
		Ok(())
	}
}
