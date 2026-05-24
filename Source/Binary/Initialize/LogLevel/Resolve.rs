//! `LogLevel::Resolve`

use log::LevelFilter;

/// Resolve the application log level.
///
/// Resolves the log level from the RUST_LOG environment variable,
/// falling back to platform-appropriate defaults.
///
/// # Returns
///
/// Returns the resolved log level.
pub fn Fn() -> LevelFilter {
	let EnvLogLevel = std::env::var("RUST_LOG").ok().and_then(|S| s.parse::<LevelFilter>().ok());

	let DefaultLogLevel = if cfg!(debug_assertions) { LevelFilter::Debug } else { LevelFilter::Info };

	EnvLogLevel.unwrap_or(DefaultLogLevel)
}
