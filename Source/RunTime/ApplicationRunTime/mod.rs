//! Echo-scheduler-powered runtime that executes `ActionEffect` pipelines.
//! Method-per-file impls live as siblings under `RunTime/Execute/` and
//! `RunTime/Shutdown/`. The struct stays here (no `pub use` indirection)
//! so callers spell `RunTime::ApplicationRunTime::ApplicationRunTime`.
pub mod Create;

use std::sync::Arc;

use CommonLibrary::Environment::{Environment::Environment, HasEnvironment::HasEnvironment};
use Echo::Scheduler::Scheduler::Scheduler;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

#[derive(Clone)]
pub struct ApplicationRunTime {
	/// Shared handle to the application's central scheduler.
	pub Scheduler:Arc<Scheduler>,

	/// Shared handle to the `MountainEnvironment` capability provider.
	pub Environment:Arc<MountainEnvironment>,
}

#[derive(Debug, Clone)]
pub struct Struct;
