//! `ApplicationRunTime::Create`

use std::sync::Arc;

use CommonLibrary::Environment::{Environment::Environment, HasEnvironment::HasEnvironment};
use Echo::Scheduler::Scheduler::Scheduler;

use super::Struct;
use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub fn Fn(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Struct {
	dev_log!("lifecycle", "new Echo-based instance created");

	Self { Scheduler, Environment }
}
