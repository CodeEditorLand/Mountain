#![allow(non_snake_case)]

//! Inbound hover request DTO: document URI + cursor position.

use serde::{Deserialize, Serialize};

use crate::Command::Hover::Interface::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub uri:String,

	pub position:Position::Struct,
}
