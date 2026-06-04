//! Establish a gRPC connection to a Cocoon sidecar with exponential
//! back-off retry. On success initialises the per-connection metadata
//! tracked by `Shared::CONNECTION_METADATA`.

use crate::Vine::Error::VineError;

pub async fn Fn(SideCarIdentifier:String, Address:String) -> Result<(), VineError> {

	::Vine::Client::ConnectToSideCar::Fn(SideCarIdentifier, Address).await
}
