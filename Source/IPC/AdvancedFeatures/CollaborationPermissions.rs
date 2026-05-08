#![allow(non_snake_case)]

//! Per-session capability flags for the realtime collaboration
//! surface. The four bits gate edit / view / comment / share
//! actions on a `CollaborationSession::Struct`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub can_edit:bool,

	pub can_view:bool,

	pub can_comment:bool,

	pub can_share:bool,
}
