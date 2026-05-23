
//! Construct an Echo scheduler from a custom `SchedulerConfig::Struct`.
//! Returns an `Arc<Scheduler>` ready for use; emits lifecycle dev-log
//! lines with feature-gated detail (Telemetry, Debug).

use std::sync::Arc;

use Echo::Scheduler::Scheduler::Scheduler;

use crate::{
	Binary::Initialize::RuntimeBuild::{CreateBuilder, SchedulerConfig},
	dev_log,
};

pub fn Fn(Config:SchedulerConfig::Struct) -> Arc<Scheduler> {
	dev_log!("lifecycle", "[RuntimeBuild] Initializing scheduler with config: {:?}", Config);

	let Builder = CreateBuilder::Fn(Config);

	let SchedulerInstance = Builder.Build();

	#[cfg(feature = "Telemetry")]
	dev_log!("lifecycle", "[RuntimeBuild] Task metrics enabled");

	#[cfg(feature = "Debug")]
	dev_log!("lifecycle", "[RuntimeBuild] Scheduler debugging enabled");

	dev_log!("lifecycle", "[RuntimeBuild] Scheduler initialized successfully");

	Arc::new(SchedulerInstance)
}
