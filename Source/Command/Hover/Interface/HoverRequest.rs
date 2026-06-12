//! Inbound hover request DTO: document URI + cursor position.

use serde::{Deserialize, Serialize};

use crate::Command::Hover::Interface::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub uri:String,

	pub position:Position::Struct,
}
