
//! Realtime collaboration session record - id, participant
//! list, currently-active document URIs, last activity tick,
//! and the permission slot.

use serde::{Deserialize, Serialize};

use crate::IPC::AdvancedFeatures::CollaborationPermissions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub session_id:String,

	pub participants:Vec<String>,

	pub active_documents:Vec<String>,

	pub last_activity:u64,

	pub permissions:CollaborationPermissions::Struct,
}
