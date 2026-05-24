pub mod GetAll;
pub mod Get;
pub mod AddOrUpdate;
pub mod Remove;
pub mod Clear;
pub mod Count;
pub mod Contains;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

/// Output channels state containing channels by ID.
#[derive(Clone)]
pub struct Struct {
	/// Output channels organized by ID.
	pub OutputChannels:Arc<StandardMutex<HashMap<String, OutputChannelStateDTO>>>,
}
