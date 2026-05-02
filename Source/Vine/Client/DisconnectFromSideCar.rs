#![allow(non_snake_case)]

//! Disconnect from a sidecar process. Removes the entry from both the
//! connection pool and the metadata tracker.

use crate::{
	Vine::{
		Client::Shared::{CONNECTION_METADATA, SIDECAR_CLIENTS},
		Error::VineError,
	},
	dev_log,
};

pub fn Fn(SideCarIdentifier:String) -> Result<(), VineError> {
	let mut Pool = SIDECAR_CLIENTS.lock();

	if Pool.remove(&SideCarIdentifier).is_some() {
		CONNECTION_METADATA.lock().remove(&SideCarIdentifier);
		dev_log!("grpc", "[VineClient] Disconnected from sidecar '{}'", SideCarIdentifier);
		Ok(())
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier))
	}
}
