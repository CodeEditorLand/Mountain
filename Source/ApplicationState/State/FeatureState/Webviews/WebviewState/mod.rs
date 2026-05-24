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
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

/// Active webviews state containing webviews by ID.
#[derive(Clone)]
pub struct Struct {
	/// Active webviews organized by ID.
	pub ActiveWebviews:Arc<StandardMutex<HashMap<String, WebviewStateDTO>>>,
}
