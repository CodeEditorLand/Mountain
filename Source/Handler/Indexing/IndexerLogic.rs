use std::{path::PathBuf, sync::Arc};

use Common::{
	effect::{ActionEffect, AppRuntime},
	error::CommonError,
	fs::ReadFile,
};
use log::info;

/// @module IndexerLogic
/// @description Contains the logic for the file indexing task.
use super::{IndexerEnvironment::IndexerEnvironment, IndexerRuntime::IndexerRuntime};
use crate::environment::MountainEnvironment;

// This effect is generic over any runtime whose environment can provide
// FsReader.
fn CreateIndexingTask<Runtime>() -> ActionEffect<Arc<Runtime>, CommonError, ()>
where
	Runtime: AppRuntime + Send + Sync + 'static,
	Runtime::EnvironmentType: Common::environment::Requires<Arc<dyn Common::fs::FsReader>>, {
	// A simplified indexing task that just reads one file.
	ReadFile(PathBuf::from("path/to/index.file")).map(|_content| info!("[Indexer] Indexing task complete."))
}

/// A top-level effect that creates a specialized runtime and executes an
/// indexing task.
///
/// This would be called from a higher-level orchestrator in the main
/// application.
pub fn StartIndexingEffect(MainEnvironment:Arc<MountainEnvironment>) {
	tokio::spawn(async move {
		// 1. Create the specialized, limited environment.
		let IndexerEnv = Arc::new(IndexerEnvironment::New(MainEnvironment));

		// 2. Create the specialized runtime with the limited environment.
		let IndexerRuntime = IndexerRuntime::New(IndexerEnv);

		// 3. Create the indexing task.
		let IndexingTask = CreateIndexingTask();

		// 4. Run the task using the specialized runtime.
		info!("[Indexer] Spawning background indexing task.");
		if let Err(e) = IndexerRuntime.Run(IndexingTask).await {
			info!("[Indexer] Indexing task failed: {:?}", e);
		}
	});
}
