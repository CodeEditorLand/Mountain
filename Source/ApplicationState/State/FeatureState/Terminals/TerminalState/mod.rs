pub mod GetNextTerminalIdentifier;
pub mod GetAll;
pub mod Get;
pub mod GetArc;
pub mod AddOrUpdate;
pub mod Remove;
pub mod Clear;
pub mod Count;
pub mod Contains;

use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},
};
use crate::{ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO, dev_log};

/// Active terminals state containing terminals by ID with next identifier
/// counter.
#[derive(Clone)]
pub struct Struct {
	/// Active terminals organized by ID.
	pub ActiveTerminals:Arc<StandardMutex<HashMap<u64, Arc<StandardMutex<TerminalStateDTO>>>>>,

	/// Counter for generating unique terminal identifiers.
	pub NextTerminalIdentifier:Arc<AtomicU64>,
}
