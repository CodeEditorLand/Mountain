// @module IndexerLogic
// @description Contains the logic for the file indexing task. This is an
// example of a long-running background process.

use std::{path::PathBuf, sync::Arc};

use Common::{
	effect::{ActionEffect, ApplicationRunTime},
	error::CommonError,
	fs::FileSystemReader,
};
use log::info;

use super::{IndexerEnvironment::IndexerEnvironment, IndexerRunTime::IndexerRunTime};
use crate::Environment::MountainEnvironment;

/// This effect is generic over any RunTime whose Environment can provide
/// FileSystemReader.
fn create_indexing_task<RT>() -> ActionEffect<Arc<RT>, CommonError, ()>
where
	RT: ApplicationRunTime + Send + Sync + 'static,
	RT::EnvironmentType: Common::Environment::Requires<Arc<dyn FileSystemReader>>, {
	// A simplified indexing task that just reads one file.
	let effect = Common::fs::ReadFile(PathBuf::from("path/to/index.file"));
	effect.map(|_content| {
		info!("[Indexer] Indexing task complete.");
	})
}

/// A top-level effect that creates a specialized RunTime and executes an
/// indexing task.
///
/// This would be called from a higher-level orchestrator in the main
/// application.
pub fn StartIndexingEffect(main_Environment:Arc<MountainEnvironment>, main_runtime:Arc<IndexerRunTime>) {
	tokio::spawn(async move {
		// 1. Create the specialized, limited Environment.
		let indexer_env = Arc::new(IndexerEnvironment::New(main_Environment));

		// 2. Create the specialized RunTime with the limited Environment.
		// In our architecture, we can reuse the main runtime but pass the limited env.
		// For the sake of demonstrating the pattern, we'll imagine a new runtime.
		let indexer_runtime = IndexerRunTime {
			Scheduler:main_runtime.Scheduler.clone(),
			Environment:Arc::new(crate::Environment::MountainEnvironment::New(
				indexer_env.MainEnvironment.ApplicationHandle.clone(),
			)), /* This is conceptually tricky; a better approach would be for the runtime to take a generic
			     * Environment. */
		};

		// 3. Create the indexing task.
		let indexing_task = create_indexing_task();

		// 4. Run the task using the specialized RunTime.
		info!("[Indexer] Spawning background indexing task.");
		if let Err(e) = indexer_runtime.Run(indexing_task).await {
			info!("[Indexer] Indexing task failed: {:?}", e);
		}
	});
}
