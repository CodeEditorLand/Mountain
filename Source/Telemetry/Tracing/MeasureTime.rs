
//! `measure_time!($name, { … })` - record the elapsed time of a block as
//! a `dev_log` line under the lifecycle tag. No-op when the `Telemetry`
//! feature is disabled.
//!
//! Must be a macro (not a function) because it needs to capture the
//! caller's expression scope.

#[cfg(feature = "Telemetry")]
#[macro_export]
macro_rules! measure_time {
	($name:expr, $block:block) => {{
		let __Start = std::time::Instant::now();

		let __Result = $block;

		let __Duration = __Start.elapsed();

		$crate::dev_log!("lifecycle", "{} took {:?}", $name, __Duration);

		__Result
	}};
}

#[cfg(not(feature = "Telemetry"))]
#[macro_export]
macro_rules! measure_time {
	($name:expr, $block:block) => {{ $block }};
}
