#![allow(non_snake_case)]

//! Echo-scheduler-powered runtime that executes `ActionEffect` pipelines.
//! Method-per-file impls live as siblings under `RunTime/Execute/` and
//! `RunTime/Shutdown/`. The struct stays here (no `pub use` indirection)
//! so callers spell `RunTime::ApplicationRunTime::ApplicationRunTime`.

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

impl ApplicationRunTime {
	pub fn Create(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		dev_log!("lifecycle", "new Echo-based instance created");
		Self { Scheduler, Environment }
	}
}

impl HasEnvironment for ApplicationRunTime {
	type EnvironmentType = MountainEnvironment;

	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }
}

impl Environment for ApplicationRunTime {}
