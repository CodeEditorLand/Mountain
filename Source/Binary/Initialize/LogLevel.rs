//! # LogLevel
//!
//! Resolves the logging level for the application.
//!
//! ## RESPONSIBILITIES
//!
//! ### Log Level Resolution
//! - Read RUST_LOG environment variable
//! - Apply default log level based on build type
//! - Resolve final log level for logging initialization
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides log level configuration
//!
//! ### Dependencies
//! - log: Logging framework
//! - std::env: Environment variable access
//!
//! ### Dependents
//! - Fn() main entry point: Uses resolved log level
//!
//! ## SECURITY
//!
//! ### Considerations
//! - No security impact (logging only)
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Log level resolution is fast
//! - Correct log level reduces logging overhead

use log::LevelFilter;

/// Resolve the application log level.
///
/// Resolves the log level from the RUST_LOG environment variable,
/// falling back to platform-appropriate defaults.
///
/// # Returns
///
/// Returns the resolved log level.
pub fn Resolve() -> LevelFilter {
	let EnvLogLevel = std::env::var("RUST_LOG")
		.ok()
		.and_then(|s| s.parse::<LevelFilter>().ok());

	let DefaultLogLevel = if cfg!(debug_assertions) {
		LevelFilter::Debug
	} else {
		LevelFilter::Info
	};

	EnvLogLevel.unwrap_or(DefaultLogLevel)
}

/// Get the default log level for the current build type.
///
/// Returns the default log level based on whether this is a debug
/// or release build.
///
/// # Returns
///
/// Returns the default log level.
pub fn GetDefault() -> LevelFilter {
	if cfg!(debug_assertions) {
		LevelFilter::Debug
	} else {
		LevelFilter::Info
	}
}
