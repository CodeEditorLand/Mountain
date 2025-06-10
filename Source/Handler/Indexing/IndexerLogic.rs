use std::{path::PathBuf, sync::Arc};

use Common::{
	effect::{ActionEffect, ApplicationRunTime},
	error::CommonError,
	fs::ReadFile,
};
use log::info;

// @module IndexerLogic
// @description Contains the logic for the file indexing task.
use super::{IndexerEnvironment::IndexerEnvironment, IndexerRunTime::IndexerRunTime};
use crate::environment::MountainEnvironment;

// This effect is generic over any RunTime whose environment can provide
// FileSystemReader.
fn CreateIndexingTask<RunTime>() -> ActionEffect<Arc<RunTime>, CommonError, ()>
where
	RunTime: ApplicationRunTime + Send + Sync + 'static,
	RunTime::EnvironmentType: Common::environment::Requires<Arc<dyn Common::fs::FileSystemReader>>, {
	// A simplified indexing task that just reads one file.
	ReadFile(PathBuf::from("path/to/index.file")).map(|_content| info!("[Indexer] Indexing task complete."))
}

// A top-level effect that creates a specialized RunTime and executes an
// indexing task.
//
// This would be called from a higher-level orchestrator in the main
// application.
pub fn StartIndexingEffect(MainEnvironment:Arc<MountainEnvironment>) {
	tokio::spawn(async move {
		// 1. Create the specialized, limited environment.
		let IndexerEnv = Arc::new(IndexerEnvironment::New(MainEnvironment));

		// 2. Create the specialized RunTime with the limited environment.
		let IndexerRunTime = IndexerRunTime::New(IndexerEnv);

		// 3. Create the indexing task.
		let IndexingTask = CreateIndexingTask();

		// 4. Run the task using the specialized RunTime.
		info!("[Indexer] Spawning background indexing task.");
		if let Err(e) = IndexerRunTime.Run(IndexingTask).await {
			info!("[Indexer] Indexing task failed: {:?}", e);
		}
	});
}
