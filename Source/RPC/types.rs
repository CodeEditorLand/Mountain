//! # Shared RPC Types
//!
//! Shared type definitions for RPC services.

// Placeholder for shared RPC types
// This module will contain common types used across RPC services

use serde::{Deserialize, Serialize};

/// Common request/response structures
pub mod common {
	use super::*;

	pub struct Request<T> {
		pub data:T,
	}

	pub struct Response<T> {
		pub data:T,
	}
}
