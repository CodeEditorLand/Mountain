
//! Default scheduler bring-up: forward to `BuildWithConfig::Fn` with
//! `SchedulerConfig::Struct::default()`. The default adapts to the
//! active build profile (CPU count workers, telemetry under
//! `Telemetry`, log level by `Debug` / `Development`).

use std::sync::Arc;

use Echo::Scheduler::Scheduler::Scheduler;

use crate::Binary::Initialize::RuntimeBuild::{BuildWithConfig, SchedulerConfig};

pub fn Fn() -> Arc<Scheduler> { BuildWithConfig::Fn(SchedulerConfig::Struct::default()) }
