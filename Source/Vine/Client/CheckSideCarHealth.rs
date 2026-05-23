//! Health check: connection exists in the pool, last activity within
//! `HEALTH_CHECK_INTERVAL_MS`, and failure count below the retry
//! threshold.

use std::time::Duration;

use crate::Vine::{
	Client::Shared::{CONNECTION_METADATA, HEALTH_CHECK_INTERVAL_MS, MAX_RETRY_ATTEMPTS},
	Error::VineError,
};

pub fn Fn(SideCarIdentifier:&str) -> Result<bool, VineError> {
	let Metadata = CONNECTION_METADATA.lock();

	if let Some(Connection) = Metadata.get(SideCarIdentifier) {
		let IsStale = Connection.LastActivity.elapsed() > Duration::from_millis(HEALTH_CHECK_INTERVAL_MS);

		let HasManyFailures = Connection.FailureCount > MAX_RETRY_ATTEMPTS;

		Ok(Connection.IsHealthy && !IsStale && !HasManyFailures)
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()))
	}
}
