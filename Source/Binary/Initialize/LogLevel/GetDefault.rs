//! `LogLevel::GetDefault`

use log::LevelFilter;

/// Get the default log level for the current build type.
///
/// Returns the default log level based on whether this is a debug
/// or release build.
///
/// # Returns
///
/// Returns the default log level.
pub fn Fn() -> LevelFilter { if cfg!(debug_assertions) { LevelFilter::Debug } else { LevelFilter::Info } }
