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
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

/// Active tree views state containing tree views by ID.
#[derive(Clone)]
pub struct Struct {
	/// Active tree views organized by ID.
	pub ActiveTreeViews:Arc<StandardMutex<HashMap<String, TreeViewStateDTO>>>,
}
