pub mod GetAll;
pub mod Get;
pub mod SetAll;
pub mod AddOrUpdate;
pub mod Remove;
pub mod Clear;
pub mod Count;
pub mod Contains;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

/// Scanned extensions containing discovered extension metadata.
#[derive(Clone)]
pub struct Struct {
	/// Scanned extensions by identifier.
	pub ScannedExtensions:Arc<StandardMutex<HashMap<String, ExtensionDescriptionStateDTO>>>,
}
