// @module IndexerRunTime
// @description Defines the specialized RunTime for executing indexing effects.
// This demonstrates a security boundary by limiting the capabilities available
// to a task.

use super::IndexerEnvironment::IndexerEnvironment;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// A type alias for a specialized RunTime that uses the `IndexerEnvironment`.
///
/// Any `ActionEffect` run with this RunTime can only access the capabilities
/// defined by `IndexerEnvironment` (e.g., read-only filesystem), preventing it
/// from accessing the UI, terminals, or other sensitive APIs.
pub type IndexerRunTime = ApplicationRunTime; // In our new model, the runtime is generic
// and the Environment provides the capability constraints.
