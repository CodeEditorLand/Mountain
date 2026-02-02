//! # Logging Plugin Module
//!
//! Configures and creates the Tauri logging plugin with appropriate targets and
//! filters.

use log::LevelFilter;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

/// Creates and configures the logging plugin with multi-target output and level
/// filtering.
///
/// # Arguments
///
/// * `LogLevel` - The desired log level (Trace, Debug, Info, Warn, Error)
///
/// # Returns
///
/// A configured `tauri_plugin_log::TauriPlugin` instance.
///
/// # Logging Strategy
///
/// - Release default: Info (low noise) unless RUST_LOG overrides
/// - Debug default: Debug (high fidelity) unless RUST_LOG overrides
/// - Very noisy dependencies are capped using level_for(...) and filter(...)
///
/// # Targets
///
/// - Stdout: Console output for development/terminal viewing
/// - LogDir: Persistent log file (Mountain.log) in the app's log directory
/// - Webview: Logs sent to the webview console for frontend debugging
///
/// # Noise Filtering
///
/// The following noisy dependencies are capped at Info level regardless of
/// RUST_LOG:
/// - hyper: HTTP library verbose logs
/// - mio: Async I/O polling logs
/// - tao: Windowing system logs
/// - tracing: Structured logging internal logs
///
/// Additionally, the following targets are filtered out entirely:
/// - polling: File watcher events (very noisy)
/// - tokio_reactor: Async reactor events
/// - want: Connection readiness logs
pub fn LoggingPlugin(LogLevel:LevelFilter) -> tauri_plugin_log::TauriPlugin {
	tauri_plugin_log::Builder::new()
		// Configure output targets
		.targets([
			Target::new(TargetKind::Stdout),
			Target::new(TargetKind::LogDir {
				file_name: Some("Mountain.log".into()),
			}),
			Target::new(TargetKind::Webview),
		])
		// Configure file rotation and timezone
		.timezone_strategy(TimezoneStrategy::UseLocal)
		.rotation_strategy(RotationStrategy::KeepAll)
		// Set base log level
		.level(LogLevel)
		// Cap very noisy dependencies at Info level
		.level_for("hyper", LevelFilter::Info)
		.level_for("mio", LevelFilter::Info)
		.level_for("tao", LevelFilter::Info)
		.level_for("tracing", LevelFilter::Info)
		// Filter out extremely noisy targets
		.filter(|Metadata| {
			!Metadata.target().starts_with("polling")
				&& !Metadata.target().starts_with("tokio_reactor")
				&& !Metadata.target().starts_with("want")
		})
		// Format logs with category-like structure: [LEVEL] [TARGET] message
		.format(|out, message, record| {
			out.finish(format_args!(
				"[{:<5}] [{}] {}",
				record.level(),
				record.target(),
				message
			))
		})
		.build()
}
