#![allow(non_snake_case)]

//! Wind profile-state slice: every known profile, the home
//! directory, and the active profile descriptor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub all:Vec<serde_json::Value>,

	pub home:String,

	pub profile:serde_json::Value,
}
