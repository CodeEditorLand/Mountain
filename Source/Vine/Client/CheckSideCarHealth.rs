//! Health check: connection exists in the pool, last activity within
//! `HEALTH_CHECK_INTERVAL_MS`, and failure count below the retry
//! threshold.

use crate::Vine::Error::VineError;

pub fn Fn(SideCarIdentifier:&str) -> Result<bool, VineError> {
	::Vine::Client::CheckSideCarHealth::Fn(SideCarIdentifier)
}
