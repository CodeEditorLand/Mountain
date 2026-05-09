#![allow(non_snake_case)]

//! Single-worker scheduler for debugging. Predictable execution order
//! makes step-through and trace inspection tractable. Compile-time
//! gated to the `Debug` feature.

#[cfg(feature = "Debug")]
use std::sync::Arc;

#[cfg(feature = "Debug")]
use Echo::Scheduler::Scheduler::Scheduler;

#[cfg(feature = "Debug")]
use crate::{
	Binary::Initialize::RuntimeBuild::{BuildWithConfig, SchedulerConfig},
	dev_log,
};

#[cfg(feature = "Debug")]
pub fn Fn() -> Arc<Scheduler> {

	dev_log!("lifecycle", "[RuntimeBuild] Creating debug scheduler (single-threaded)");

	BuildWithConfig::Fn(SchedulerConfig::Struct { WorkerCount:Some(1), ..Default::default() })
}
