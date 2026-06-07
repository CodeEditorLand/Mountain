//! # Logging Plugin Module
//!
//! Configures and creates the Tauri logging plugin with appropriate targets and
//! filters.

use log::LevelFilter;
use tauri::plugin::TauriPlugin;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

use crate::IPC::DevLog;

/// Compress a Rust module target path to its final segment.
///
/// `D::Binary::Main::Entry` → `Entry`
/// `D::Environment::StorageProvider` → `StorageProvider`
fn CompressTarget(Target:&str) -> &str { Target.rsplit("::").next().unwrap_or(Target) }

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
/// # Short Mode
///
/// When `Trace=short`:
/// - Module targets compressed to last segment
/// - Long app-data paths aliased to `$APP`
/// - Storage key-by-key logs suppressed (batch count only)
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
pub fn LoggingPlugin<R:tauri::Runtime>(LogLevel:LevelFilter) -> TauriPlugin<R> {
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

		// `ignore` and `globset` (used by Mountain's extension scanner +
		// `WorkspaceProvider::FindFiles` walk) emit a DEBUG line per
		// `.gitignore` opened, per glob compiled, and per file
		// whitelisted/ignored. A single `debug-electron-bundled` boot
		// produces tens of thousands of these lines, drowning out every
		// extension activation / SCM register / GIT-MARK / IPC trace
		// the rest of Mountain emits at the same level. Cap to Warn so
		// the extension-activation signal is readable.
		.level_for("ignore", LevelFilter::Warn)
		.level_for("ignore::walk", LevelFilter::Warn)
		.level_for("ignore::gitignore", LevelFilter::Warn)
		.level_for("globset", LevelFilter::Warn)

		// `keyring` (used by Mountain's secret-storage path on the
		// `dev1phpTools.license.data` lookup chain) emits a 3-line
		// DEBUG block per `get_password` call - "creating entry",
		// "created entry", "get password from entry" - per refresh
		// tick. After the workbench paints these fire indefinitely.
		// Cap to Warn alongside the other dependency mutes.
		.level_for("keyring", LevelFilter::Warn)

		// Tauri's asset manager logs every fallback probe (`asset.html`,
		// `asset/index.html`, etc.) at DEBUG and then a single ERROR
		// when the asset is not in the bundled resources. Land serves
		// every workbench asset from the Astro dev server at
		// `localhost:21100`; the requests reaching `tauri::manager`
		// are sourcemap (`.js.map`, `.wasm.map`) and module URLs that
		// WebKit auto-fetches with the wrong base because the worker /
		// importmap is rooted at `tauri://localhost`. The 404 is
		// expected and harmless. Cap the chatter to Warn so the
		// extension-activation signal stays readable.
		.level_for("tauri::manager", LevelFilter::Warn)
		.level_for("tauri::manager::asset", LevelFilter::Warn)
		.level_for("tauri::webview", LevelFilter::Info)
		.level_for("wry", LevelFilter::Info)

		// Filter out extremely noisy targets
		.filter(|Metadata| {
			!Metadata.target().starts_with("polling")

				&& !Metadata.target().starts_with("tokio_reactor")

				&& !Metadata.target().starts_with("want")
		})

		// Format logs with category-like structure: [LEVEL] [TARGET] message
		.format(|out, message, record| {
			if DevLog::IsShort::Fn() {
				let ShortTarget = CompressTarget(record.target());

				let RawMessage = format!("{}", message);

				let Aliased = DevLog::AliasPath::Fn(&RawMessage);

				out.finish(format_args!(
					"[{:<5}] [{}] {}",

					record.level(),

					ShortTarget,

					Aliased
				))
			} else {
				out.finish(format_args!(
					"[{:<5}] [{}] {}",

					record.level(),

					record.target(),

					message
				))
			}
		})
		.build()
}
