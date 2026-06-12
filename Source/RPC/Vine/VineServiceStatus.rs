//! Vine gRPC service health enum.
use serde::{Deserialize, Serialize};

/// Vine gRPC service connection status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	Connected,

	Disconnected,

	Error,
}
