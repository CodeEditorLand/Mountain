pub mod GetAll;
pub mod Get;
pub mod AddOrUpdate;
pub mod Remove;
pub mod Clear;
pub mod Count;
pub mod Contains;
pub mod GetURIs;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

/// Open documents state containing documents by URI.
#[derive(Clone)]
pub struct Struct {
	/// Open documents organized by URI.
	pub OpenDocuments:Arc<StandardMutex<HashMap<String, DocumentStateDTO>>>,
}
