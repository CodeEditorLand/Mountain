#![allow(non_snake_case)]

//! Per-controller registration record. Carries the extension-provided
//! identifier, label, owning sidecar, active flag, and supported test
//! type tags. Stored in `TestProviderState::Struct` keyed by
//! `ControllerIdentifier`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub ControllerIdentifier:String,

	pub Label:String,

	pub SideCarIdentifier:Option<String>,

	pub IsActive:bool,

	pub SupportedTestTypes:Vec<String>,
}
