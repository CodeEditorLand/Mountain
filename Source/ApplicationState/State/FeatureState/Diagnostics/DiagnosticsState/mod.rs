pub mod GetAll;
pub mod GetByOwner;
pub mod GetByOwnerAndResource;
pub mod SetByOwner;
pub mod SetByOwnerAndResource;
pub mod ClearByOwner;
pub mod ClearByOwnerAndResource;
pub mod ClearAll;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

/// Diagnostic errors state containing markers by owner and resource.
#[derive(Clone)]
pub struct Struct {
	/// Diagnostics map organized by owner and resource URI.
	///
	/// Structure: owner -> resource URI -> list of markers
	pub DiagnosticsMap:Arc<StandardMutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,
}
