/// @module IndexerRuntime
/// @description Defines the specialized runtime for executing indexing effects.
use super::IndexerEnvironment::IndexerEnvironment;
use crate::runtime::DefaultRuntime::DefaultRuntime; // Using the simpler runtime for this example

/// A type alias for a specialized runtime that uses the `IndexerEnvironment`.
/// Any `ActionEffect` run with this runtime can only access the `FsReader`
/// capability.
pub type IndexerRuntime = DefaultRuntime<IndexerEnvironment>;
