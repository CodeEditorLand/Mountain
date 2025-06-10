// @module IndexerRunTime
// @description Defines the specialized RunTime for executing indexing effects.
use super::IndexerEnvironment::IndexerEnvironment;
use crate::RunTime::DefaultRunTime::DefaultRunTime; // Using the simpler RunTime for this example

// A type alias for a specialized RunTime that uses the `IndexerEnvironment`.
// Any `ActionEffect` run with this RunTime can only access the
// `FileSystemReader` capability.
pub type IndexerRunTime = DefaultRunTime<IndexerEnvironment>;
