#![allow(non_snake_case)]

//! Connectivity slice of `IPCStatusReport::Struct` - is the
//! IPC server reachable, when did it last heartbeat, and how
//! long has the current connection been alive.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub is_connected:bool,
	pub last_heartbeat:u64,
	pub connection_duration:u64,
}
