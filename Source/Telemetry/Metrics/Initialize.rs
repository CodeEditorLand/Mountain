//! No-op initialiser kept so `Telemetry::Fn::Fn` can call into
//! Metrics symmetrically with Tracing. Real bring-up will hydrate the
//! registry from `MountainEnvironment`.

#[cfg(feature = "Telemetry")]
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> {
	use crate::dev_log;

	dev_log!("metrics", "metrics system initialized");

	Ok(())
}

#[cfg(not(feature = "Telemetry"))]
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
