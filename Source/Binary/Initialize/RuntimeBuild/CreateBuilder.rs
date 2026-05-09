#![allow(non_snake_case)]

//! Translate a `SchedulerConfig::Struct` into a configured Echo
//! `SchedulerBuilder`. Worker counts are clamped to `[1, 256]` so the
//! caller can't accidentally request 0 workers or oversaturate.

use Echo::Scheduler::SchedulerBuilder::SchedulerBuilder;

use crate::{Binary::Initialize::RuntimeBuild::SchedulerConfig, dev_log};

pub fn Fn(Config:SchedulerConfig::Struct) -> SchedulerBuilder {
	let mut Builder = SchedulerBuilder::Create();

	if let Some(Count) = Config.WorkerCount {
		let Count = Count.clamp(1, 256);

		Builder = Builder.WithWorkerCount(Count);

		dev_log!("lifecycle", "[RuntimeBuild] Configuring {} worker threads", Count);
	}

	Builder
}
