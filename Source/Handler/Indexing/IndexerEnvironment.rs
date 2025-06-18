// @module IndexerEnvironment
// @description Defines a specialized, limited-capability Environment for
// indexing tasks, demonstrating a security and safety pattern.

use std::sync::Arc;

use Common::{
	Environment::{Environment, Requires},
	fs::FileSystemReader,
};

use crate::Environment::MountainEnvironment;

/// A specialized Environment that only provides read-only filesystem access.
///
/// By using this Environment, indexing tasks are prevented at compile time from
/// accessing any other capabilities like UI, IPC, or file writing. This
/// improves security and makes the task's behavior more predictable.
#[derive(Clone)]
pub struct IndexerEnvironment {
	// It wraps the main application Environment to delegate the FileSystemReader implementation.
	pub MainEnvironment:Arc<MountainEnvironment>,
}

impl IndexerEnvironment {
	pub fn New(main_Environment:Arc<MountainEnvironment>) -> Self { Self { MainEnvironment:main_Environment } }
}

impl Environment for IndexerEnvironment {}

// --- Capability Implementation ---
// It only provides one capability: FileSystemReader.
impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for IndexerEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> {
		// It fulfills the requirement by delegating to the main Environment.
		self.MainEnvironment.Require()
	}
}
