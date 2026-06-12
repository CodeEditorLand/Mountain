//! Vine gRPC connection info DTO.
use serde::{Deserialize, Serialize};

/// Vine gRPC connection info: identifies a remote gRPC service by name and
/// endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub service_name:String,

	pub endpoint:String,
}
