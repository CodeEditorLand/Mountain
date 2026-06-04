//! Process-wide singleton storing every runtime gate enabled by Cargo
//! features at boot. The set is populated lazily on first read and is
//! cheap to consult thereafter (`HashSet::contains`).

use std::{collections::HashSet, sync::OnceLock};

pub(crate) static GATES:OnceLock<HashSet<String>> = OnceLock::new();

pub(crate) fn Initialise() -> &'static HashSet<String> {
	GATES.get_or_init(|| {
		let mut Gates = HashSet::new();

		#[cfg(feature = "Debug")]
		{
			Gates.insert("verbose-logging".to_string());

			Gates.insert("performance-profiling".to_string());

			Gates.insert("detailed-error-messages".to_string());

			Gates.insert("experimental-features".to_string());
		}

		#[cfg(feature = "Development")]
		{
			Gates.insert("development-tools".to_string());

			Gates.insert("workspace-diagnostics".to_string());

			Gates.insert("extension-hot-reload".to_string());
		}

		#[cfg(feature = "Telemetry")]
		{
			Gates.insert("tracing".to_string());

			Gates.insert("metrics".to_string());

			Gates.insert("performance-monitoring".to_string());
		}

		Gates
	})
}
