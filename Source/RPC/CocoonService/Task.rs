//! Task-domain handlers for `CocoonService`.
//! `ExecuteTask::Fn`, `RegisterTaskProvider::Fn`, `TerminateTask::Fn`.
/// ExecuteTask handler: executes a registered task.
pub mod ExecuteTask;

/// RegisterTaskProvider handler: registers a task provider with the
/// environment.
pub mod RegisterTaskProvider;

/// TerminateTask handler: terminates a running task.
pub mod TerminateTask;
