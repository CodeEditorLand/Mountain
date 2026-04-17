//! # Vine RPC Types
//!
//! Re-exported Vine types for gRPC inter-service communication.

// Placeholder for Vine types
// This module will re-export types from the Vine component

use serde::{Deserialize, Serialize};

/// Vine connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineConnectionInfo {
	pub service_name:String,
	pub endpoint:String,
}

/// Vine service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VineServiceStatus {
	Connected,
	Disconnected,
	Error,
}
