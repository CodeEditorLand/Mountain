//! `Shared::ValidateMessageSize`

use std::{
	collections::HashMap,
	sync::{
		Arc,
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Instant,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use tokio::sync::Notify;

use crate::Vine::{Client::NotificationFrame, Error::VineError, Generated::cocoon_service_client::CocoonServiceClient};

pub const DEFAULT_TIMEOUT_MS:u64 = 5000;
pub const MAX_RETRY_ATTEMPTS:usize = 3;
pub const RETRY_BASE_DELAY_MS:u64 = 100;
pub const MAX_MESSAGE_SIZE_BYTES:usize = 4 * 1024 * 1024;
pub const HEALTH_CHECK_INTERVAL_MS:u64 = 30000;
pub const CONNECTION_TIMEOUT_MS:u64 = 10000;
pub const NOTIFICATION_BROADCAST_CAPACITY:usize = 4096;
static CONNECTION_NOTIFIERS:OnceLock<Arc<parking_lot::RwLock<HashMap<String, Arc<Notify>>>>> = OnceLock::new();
pub static SHUTDOWN_FLAG:AtomicBool = AtomicBool::new(false);

/// Reject messages above `MAX_MESSAGE_SIZE_BYTES` to bound the worst-case
/// gRPC frame. Mirrors tonic's own check so we don't pay the codec round-
/// trip for an oversize payload.
pub fn Fn(Data:&[u8]) -> Result<(), VineError> {
	if Data.len() > MAX_MESSAGE_SIZE_BYTES {
		Err(VineError::MessageTooLarge { ActualSize:Data.len(), MaxSize:MAX_MESSAGE_SIZE_BYTES })
	} else {
		Ok(())
	}
}
