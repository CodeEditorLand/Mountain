//! `Timer::StopAndRecord`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::Telemetry::Metrics::GlobalRegistry;

pub fn Fn(self) -> Duration {
		let Elapsed = This.Start.elapsed();

		GlobalRegistry::REGISTRY.RecordHistogram(&This.Name, Elapsed, This.Labels);

		Elapsed
	}
