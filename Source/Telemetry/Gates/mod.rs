//! # Telemetry Gates
//!
//! This module provides compile-time and runtime feature gates for controlling
//! telemetry visibility and behavior across different build configurations.

use std::{collections::HashSet, sync::OnceLock};

/// Static set of runtime-enabled feature gates
static RUNTIME_GATES:OnceLock<HashSet<String>> = OnceLock::new();

/// Check if Debug build features are enabled (compile-time)
#[inline]
pub const fn is_debug_build() -> bool { cfg!(debug_assertions) }

/// Check if Development build features are enabled (compile-time)
#[inline]
pub const fn is_development_build() -> bool { cfg!(feature = "Development") || cfg!(debug_assertions) }

/// Check if Telemetry is enabled (compile-time)
#[inline]
pub const fn is_telemetry_enabled() -> bool { cfg!(feature = "Telemetry") }

/// Check if metrics collection is enabled (compile-time)
#[inline]
pub const fn is_metrics_enabled() -> bool { cfg!(feature = "MetricsCollection") }

/// Check if distributed tracing is enabled (compile-time)
#[inline]
pub const fn is_distributed_tracing_enabled() -> bool { cfg!(feature = "DistributedTracing") }

/// Check if feature flags are enabled at compile-time
#[inline]
pub const fn is_feature_flags_enabled() -> bool { cfg!(feature = "RuntimeFeatureFlags") }

/// Get the runtime gates set
pub fn get_runtime_gates() -> &'static HashSet<String> {
	RUNTIME_GATES.get_or_init(|| {
		let mut gates = HashSet::new();

		#[cfg(feature = "Debug")]
		{
			gates.insert("verbose-logging".to_string());
			gates.insert("performance-profiling".to_string());
			gates.insert("detailed-error-messages".to_string());
			gates.insert("experimental-features".to_string());
		}

		#[cfg(feature = "Development")]
		{
			gates.insert("development-tools".to_string());
			gates.insert("workspace-diagnostics".to_string());
			gates.insert("extension-hot-reload".to_string());
		}

		#[cfg(feature = "Telemetry")]
		{
			gates.insert("tracing".to_string());
			gates.insert("metrics".to_string());
			gates.insert("performance-monitoring".to_string());
		}

		gates
	})
}

/// Check if a runtime gate is enabled
pub fn runtime_gate_enabled(gate_name:&str) -> bool { get_runtime_gates().contains(gate_name) }

/// Enable a runtime gate
pub fn enable_runtime_gate(gate_name:String) -> Result<(), String> {
	RUNTIME_GATES.get_or_init(|| HashSet::new()).insert(gate_name);
	Ok(())
}

/// List all enabled runtime gates
pub fn list_enabled_gates() -> Vec<String> { get_runtime_gates().iter().cloned().collect() }

/// Validate required gates for a feature
pub fn validate_required_gates(feature_name:&str, required_gates:&[&str]) -> Result<(), String> {
	let enabled = get_runtime_gates();

	let missing:Vec<_> = required_gates.iter().filter(|gate| !enabled.contains(*gate)).collect();

	if !missing.is_empty() {
		Err(format!("Feature '{}' requires gates: {:?}", feature_name, missing))
	} else {
		Ok(())
	}
}
