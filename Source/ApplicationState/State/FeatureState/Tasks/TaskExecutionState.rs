//! Active task execution registry keyed by a u64 run-ID generated at
//! execution time.  Each entry stores the raw task definition JSON so
//! `tasks:getTaskExecution` can return it without asking Cocoon again.

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},
};

use parking_lot::Mutex;

use serde_json::Value;

use crate::dev_log;

/// Registry of in-flight task executions.
#[derive(Clone)]
pub struct TaskExecutionState {

	/// Active executions: run-ID → task definition JSON.
	pub ActiveExecutions:Arc<Mutex<HashMap<u64, Value>>>,

	/// Monotonic counter used to stamp each new execution.
	pub NextExecutionId:Arc<AtomicU64>,
}

impl Default for TaskExecutionState {

	fn default() -> Self {
		dev_log!("task", "[TaskExecutionState] Initializing default task execution state...");

		Self {
			ActiveExecutions:Arc::new(Mutex::new(HashMap::new())),

			NextExecutionId:Arc::new(AtomicU64::new(1)),
		}
	}
}

impl TaskExecutionState {

	/// Reserves the next unique run-ID (atomic fetch-add).
	pub fn NextId(&self) -> u64 { self.NextExecutionId.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Inserts or replaces the stored definition for a run-ID.
	pub fn Insert(&self, id:u64, definition:Value) {
		let mut guard = self.ActiveExecutions.lock();

		guard.insert(id, definition);

		dev_log!("task", "[TaskExecutionState] Execution registered id={}", id);
	}

	/// Returns the stored definition for a run-ID, or `None` if not found.
	pub fn Get(&self, id:u64) -> Option<Value> { self.ActiveExecutions.lock().get(&id).cloned() }

	/// Removes a run-ID from the registry (called on task.didEnd).
	pub fn Remove(&self, id:u64) {
		let mut guard = self.ActiveExecutions.lock();

		guard.remove(&id);

		dev_log!("task", "[TaskExecutionState] Execution removed id={}", id);
	}
}
