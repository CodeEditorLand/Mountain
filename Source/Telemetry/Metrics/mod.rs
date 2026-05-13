#![allow(non_snake_case)]

//! # Performance and Operational Metrics
//!
//! Bounded ring buffer of recent metric observations. Every emit goes
//! through a process-wide `MetricsRegistry::Struct` that drops the oldest
//! sample once 10 000 entries have been recorded.
//!
//! Layout (one export per file, file name = identity):
//! - `Metric::Struct` + `MetricValue::Enum` - the observation payload.
//! - `MetricsRegistry::Struct` - the storage primitive.
//! - `Timer::Struct` - RAII-style histogram timer.
//! - `RecordCounter::Fn`, `RecordGauge::Fn`, `GetAllMetrics::Fn` - convenience
//!   accessors against the global registry.
//! - `Initialize::Fn` - no-op bring-up hook.
//! - `GlobalRegistry::REGISTRY` - module-private singleton.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. Pending wire-up from
//! `Binary::Main` and the IPC fast paths.

pub mod GetAllMetrics;

pub mod Initialize;

pub mod Metric;

pub mod MetricValue;

pub mod MetricsRegistry;

pub mod RecordCounter;

pub mod RecordGauge;

pub mod Timer;

pub(crate) mod GlobalRegistry;

#[cfg(test)]
mod tests {

	use std::collections::HashMap;

	use super::{MetricsRegistry, Timer};

	#[test]
	fn registry_creation() {
		let Registry = MetricsRegistry::Struct::new(100);

		assert!(Registry.GetAllMetrics().is_empty());
	}

	#[test]
	fn counter_recording() {
		let Registry = MetricsRegistry::Struct::new(100);

		Registry.RecordCounter("test.counter", 42.0, HashMap::new());

		let All = Registry.GetAllMetrics();

		assert_eq!(All.len(), 1);

		assert_eq!(All[0].Name, "test.counter");
	}

	#[test]
	fn gauge_recording() {
		let Registry = MetricsRegistry::Struct::new(100);

		Registry.RecordGauge("test.gauge", 99.9, HashMap::new());

		let All = Registry.GetAllMetrics();

		assert_eq!(All.len(), 1);

		assert_eq!(All[0].Name, "test.gauge");
	}

	#[test]
	fn timer_records() {
		let T = Timer::Struct::Start("test.timer");

		std::thread::sleep(std::time::Duration::from_millis(10));

		let Elapsed = T.StopAndRecord();

		assert!(Elapsed.as_millis() >= 10);
	}
}
