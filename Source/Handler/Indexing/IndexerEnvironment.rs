use std::sync::Arc;

use Common::{
	environment::{Environment, Requires},
	fs::FileSystemReader,
};

// @module IndexerEnvironment
// @description Defines a specialized, limited-capability environment for
// indexing tasks.
use crate::environment::MountainEnvironment;

// A specialized environment that only provides read-only filesystem access.
//
// By using this environment, indexing tasks are prevented at compile time from
// accessing any other capabilities like UI, IPC, or file writing.
#[derive(Clone)]
pub struct IndexerEnvironment {
	// It wraps the main application environment to delegate the FileSystemReader implementation.
	MainEnvironment:Arc<MountainEnvironment>,
}

impl IndexerEnvironment {
	pub fn New(MainEnvironment:Arc<MountainEnvironment>) -> Self { Self { MainEnvironment } }
}

impl Environment for IndexerEnvironment {}

// --- Capability Implementation ---
// It only provides one capability: FileSystemReader.
impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for IndexerEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> {
		// It fulfills the requirement by delegating to the main environment.
		self.MainEnvironment.Require()
	}
}
