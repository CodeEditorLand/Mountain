pub mod GetGlobalConfiguration;
pub mod SetGlobalConfiguration;
pub mod GetWorkspaceConfiguration;
pub mod SetWorkspaceConfiguration;
pub mod GetGlobalValue;
pub mod SetGlobalValue;
pub mod GetGlobalMemento;
pub mod SetGlobalMemento;
pub mod GetGlobalMementoValue;
pub mod SetGlobalMementoValue;
pub mod GetWorkspaceMemento;
pub mod SetWorkspaceMemento;
pub mod GetWorkspaceMementoValue;
pub mod SetWorkspaceMementoValue;
pub mod ClearWorkspaceMementoValue;
pub mod ClearGlobalMemento;
pub mod ClearWorkspaceMemento;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

/// Configuration and storage state.
#[derive(Clone)]
pub struct Struct {
	/// Merged global configuration from all sources.
	pub GlobalConfiguration:Arc<StandardMutex<serde_json::Value>>,

	/// Merged workspace configuration from all sources.
	pub WorkspaceConfiguration:Arc<StandardMutex<serde_json::Value>>,

	/// Global memento storage for crash recovery.
	pub MementoGlobalStorage:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,

	/// Workspace memento storage for crash recovery.
	pub MementoWorkspaceStorage:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,
}
