//! Disconnect from a sidecar process. Removes the entry from both the
//! connection pool and the metadata tracker.

use crate::Vine::Error::VineError;

pub fn Fn(SideCarIdentifier:String) -> Result<(), VineError> {
	::Vine::Client::DisconnectFromSideCar::Fn(SideCarIdentifier)
}
