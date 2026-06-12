//! Vine gRPC service health enum.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	Connected,

	Disconnected,

	Error,
}
