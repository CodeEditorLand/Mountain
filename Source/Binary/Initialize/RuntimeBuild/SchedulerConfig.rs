#![allow(non_snake_case, dead_code)]

//! Echo scheduler tuning knobs. `Default` adapts to the active build
//! profile: telemetry on under `Telemetry`, log level scales by
//! `Debug` / `Development`, worker count is `None` (= CPU count).

#[derive(Debug)]
pub struct Struct {
	pub WorkerCount:Option<usize>,
	pub EnableMetrics:bool,
	pub LogLevel:log::Level,
}

impl Default for Struct {
	fn default() -> Self {
		Self {
			WorkerCount:None,
			#[cfg(feature = "Telemetry")]
			EnableMetrics:true,
			#[cfg(not(feature = "Telemetry"))]
			EnableMetrics:false,
			#[cfg(feature = "Debug")]
			LogLevel:log::Level::Debug,
			#[cfg(all(feature = "Development", not(feature = "Debug")))]
			LogLevel:log::Level::Info,
			#[cfg(not(any(feature = "Debug", feature = "Development")))]
			LogLevel:log::Level::Warn,
		}
	}
}
