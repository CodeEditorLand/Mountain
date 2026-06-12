//! # AppLifecycle (Binary/Main)
//!
//! ## RESPONSIBILITIES
//!
//! Application lifecycle management for the Tauri application setup and
//! initialization. This module handles the complete setup process during the
//! Tauri setup hook, including tray initialization, command registration, IPC
//! server setup, window creation, environment configuration, and async service
//! initialization.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The AppLifecycle module is the **initialization layer** in Mountain's
//! architecture:
//!
//! ```text
//! Tauri Builder Setup ──► AppLifecycle::AppLifecycleSetup()
//!                              │
//!                              ├─► Tray Initialization
//!                              ├─► Command Registration
//!                              ├─► IPC Server Setup
//!                              ├─► Window Building
//!                              ├─► Environment Setup
//!                              ├─► Runtime Setup
//!                              └─► Async Service Initialization
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **AppLifecycleSetup()**: Main setup function orchestrating all
//!   initialization
//! - **Tray Initialization**: System tray icon with Dark/Light mode support
//! - **Command Registration**: Native command registration with application
//!   state
//! - **IPC Server**: Mountain IPC server for frontend-backend communication
//! - **Window Building**: Main application window configuration
//! - **MountainEnvironment**: Environment context for application services
//! - **ApplicationRunTime**: Runtime context with scheduler and environment
//! - **Status Reporter**: IPC status reporting initialization
//! - **Advanced Features**: Advanced IPC features initialization
//! - **Wind Sync**: Wind advanced sync initialization
//! - **Async Initialization**: Post-setup async service initialization
//!
//! ## ERROR HANDLING
//!
//! Returns `Result<(), Box<dyn std::error::Error>>` for setup errors.
//! Non-critical failures are logged but don't prevent application startup.
//! Critical failures are propagated to prevent incomplete startup.
//!
//! ## LOGGING
//!
//! Comprehensive logging at INFO level for major setup steps,
//! DEBUG level for detailed processing, and ERROR for failures.
//! All logs are prefixed with `[Lifecycle] [ComponentName]`.
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Async initialization spawned after main setup to avoid blocking
//! - Services initialized only when needed
//! - Clone operations minimized for Arc-wrapped shared state
//!
//! ## TODO
//! - [ ] Add setup progress tracking
//! - [ ] Implement setup timeout handling
//! - [ ] Add setup rollback mechanism on failure

use std::sync::Arc;

use tauri::Manager;
use Echo::Scheduler::Scheduler::Scheduler;

use crate::dev_log;
#[cfg(debug_assertions)]
use crate::Binary::Debug::WebkitServer;

/// Master "disable Land customisations" gate. Returns `true` when the
/// `Disable=true` env var is set (PascalCase, single-word, matching
/// the rest of Land's env surface in `.env.Land.Diagnostics`). When
/// enabled, Mountain skips:
///   - `WindowEvent::CloseRequested` intercept (Cmd+W routes natively)
///   - Cocoon + Air sidecar spawn
///   - The Wind / SkyBridge advanced-features registration
///   - The smoke-test gating that would otherwise activate via Sky
///
/// Code paths are NOT removed - just skipped at runtime so a clean
/// `Disable=` env var (or `Disable=false`) restores stock behaviour.
fn IsLandDisabled() -> bool {
	std::env::var("Disable")
		.map(|Value| Value.eq_ignore_ascii_case("true"))
		.unwrap_or(false)
}

use crate::{
	// Crate root imports
	ApplicationState::State::ApplicationState::ApplicationState,
	// Binary submodule imports
	Binary::Build::AppMenu::SetAppMenu,
	Binary::Build::WindowBuild::WindowBuild as WindowBuildFn,
	Binary::Extension::ExtensionPopulate::Fn as ExtensionPopulateFn,
	Binary::Extension::ScanPathConfigure::ScanPathConfigure as ScanPathConfigureFn,
	Binary::Register::AdvancedFeaturesRegister::AdvancedFeaturesRegister as AdvancedFeaturesRegisterFn,
	Binary::Register::CommandRegister::CommandRegister as CommandRegisterFn,
	Binary::Register::IPCServerRegister::IPCServerRegister as IPCServerRegisterFn,
	Binary::Register::StatusReporterRegister::StatusReporterRegister as StatusReporterRegisterFn,
	Binary::Register::WindSyncRegister::WindSyncRegister as WindSyncRegisterFn,
	Binary::Service::AirStart::Fn as AirStartFn,
	Binary::Service::CocoonStart::Fn as CocoonStartFn,
	Binary::Service::ConfigurationInitialize::Fn as ConfigurationInitializeFn,
	Binary::Service::VineStart::Fn as VineStartFn,
	Binary::Tray::EnableTray as EnableTrayFn,
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Logs a checkpoint message at TRACE level.
macro_rules! TraceStep {

	($($arg:tt)*) => {{

		dev_log!("lifecycle", $($arg)*);
	}};
}

/// Sets up the application lifecycle during Tauri initialization.
///
/// This function coordinates all setup operations:
/// 1. System tray initialization
/// 2. Native command registration
/// 3. IPC server initialization
/// 4. Main window creation
/// 5. Mountain environment setup
/// 6. Application runtime setup
/// 7. Status reporter initialization
/// 8. Advanced features initialization
/// 9. Wind advanced sync initialization
/// 10. Async post-setup initialization
///
/// # Parameters
///
/// * `app` - Mutable reference to Tauri App instance
/// * `app_handle` - Cloned Tauri AppHandle for async operations
/// * `localhost_url` - URL for the development server
/// * `scheduler` - Arc-wrapped Echo Scheduler
/// * `app_state` - Application state clone
///
/// # Returns
///
/// `Result<(), Box<dyn std::error::Error>>` - Ok on success, Err on critical
/// failure
pub fn AppLifecycleSetup(
	app:&mut tauri::App,

	app_handle:tauri::AppHandle,

	localhost_url:String,

	scheduler:Arc<Scheduler>,

	app_state:Arc<ApplicationState>,
) -> Result<(), Box<dyn std::error::Error>> {
	dev_log!("lifecycle", "[Lifecycle] [Setup] Setup hook started.");

	dev_log!("lifecycle", "[Lifecycle] [Setup] LocalhostUrl={}", localhost_url);

	crate::IPC::WindServiceHandlers::Utilities::LocalhostUrl::Set::Fn(localhost_url.clone());

	let app_handle_for_setup = app_handle.clone();

	TraceStep!("[Lifecycle] [Setup] AppHandle acquired.");

	// -------------------------------------------------------------------------
	// [UI] [Tray] Initialize System Tray
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[UI] [Tray] Initializing system tray...");

	if let Err(Error) = EnableTrayFn::enable_tray(app) {
		dev_log!("lifecycle", "error: [UI] [Tray] Failed to enable tray: {}", Error);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [Commands] Register native commands
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[Lifecycle] [Commands] Registering native commands...");

	if let Err(e) = CommandRegisterFn(&app_handle_for_setup, &app_state) {
		dev_log!("lifecycle", "error: [Lifecycle] [Commands] Failed to register commands: {}", e);
	}

	dev_log!("lifecycle", "[Lifecycle] [Commands] Native commands registered.");

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize IPC Server
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[Lifecycle] [IPC] Initializing Mountain IPC Server...");

	if let Err(e) = IPCServerRegisterFn(&app_handle_for_setup) {
		dev_log!("lifecycle", "error: [Lifecycle] [IPC] Failed to register IPC server: {}", e);
	}

	// -------------------------------------------------------------------------
	// [UI] [Window] Build main window
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[UI] [Window] Building main window...");

	let MainWindow = WindowBuildFn(app, localhost_url.clone());

	dev_log!("lifecycle", "[UI] [Window] Main window ready.");

	// Remove Undo/Redo from the native macOS Edit menu so Cmd+Z routes to
	// VS Code's Monaco keybinding handler instead of WKWebView's native
	// text-buffer undo. No-op on Windows/Linux.
	SetAppMenu(app);

	// DevTools auto-open is opt-in via the PascalCase env var
	// `Inspect=1` (or any non-empty value other than `0`). Naming
	// follows Land's single-word PascalCase verb convention -
	// see `.env.Land.Diagnostics` for the documented set.
	//
	// Auto-opening DevTools on every debug launch was the direct
	// cause of "I can't type or fire keybindings": the DevTools
	// window steals macOS keyboard focus the moment it appears, so
	// the main webview never becomes first responder and every
	// keystroke goes to DevTools (or the system menu) instead of
	// the workbench. The keybinding shortcut `Cmd+Alt+I` (Tauri's
	// default) and the right-click "Inspect" entry both still
	// work when needed.
	#[cfg(debug_assertions)]
	{
		let WantDevTools = std::env::var("Inspect")
			.map(|Value| !Value.is_empty() && Value != "0")
			.unwrap_or(false);

		if WantDevTools {
			dev_log!("lifecycle", "[UI] [Window] Inspect=1 set: opening DevTools.");

			MainWindow.open_devtools();
		} else {
			dev_log!(
				"lifecycle",
				"[UI] [Window] Debug build: DevTools auto-open suppressed (export Inspect=1 to override)."
			);
		}
	}

	#[cfg(debug_assertions)]
	{
		let enable_debug_server = std::env::var("DebugServer").map(|v| v != "0" && !v.is_empty()).unwrap_or(false);

		if enable_debug_server {
			// DebugServer values: mountain | cocoon | both | 1 (= mountain, legacy).
			// Mountain port: DebugServerPort or DebugServerPortMountain (default 9933).
			// Cocoon port: DebugServerPortCocoon (default 9934) - started inside the
			// Cocoon extension-host process from its own bootstrap path.
			dev_log!(
				"lifecycle",
				"[Debug] [Webkit] DebugServer mode={} Mountain-port={} Cocoon-port={}",
				std::env::var("DebugServer").unwrap_or_else(|_| "(unset)".into()),
				std::env::var("DebugServerPortMountain")
					.or_else(|_| std::env::var("DebugServerPort"))
					.unwrap_or_else(|_| "9933".into()),
				std::env::var("DebugServerPortCocoon").unwrap_or_else(|_| "9934".into())
			);

			WebkitServer::install(&MainWindow);
		}
	}

	// -------------------------------------------------------------------------
	// [UI] [Window] Intercept CloseRequested so Cmd+W (and the macOS app
	// menu's Window > Close item) routes through the workbench instead of
	// killing the whole window.
	//
	// On macOS, Tauri 2.x installs a default app menu that maps Cmd+W to
	// NSWindow's `performClose:`. The webview's keydown handler never gets
	// the event because the menu wins the responder chain. The result the
	// user sees: hitting Cmd+W to close a tab nukes the entire editor.
	//
	// The fix is the standard Electron-style handshake:
	//   1. Mountain prevents the close.
	//   2. Mountain emits `sky://window/close-requested` to the webview.
	//   3. Sky listens, asks the workbench to close the active editor; if there is
	//      no active editor (or the workbench refuses), Sky calls
	//      `nativeHost:closeWindow`, which uses `WebviewWindow::destroy()` to tear
	//      the window down without re-firing CloseRequested.
	if IsLandDisabled() {
		dev_log!(
			"window",
			"[UI] [Window] Disable=true: CloseRequested intercept SKIPPED (Cmd+W will close window natively)"
		);
	} else {
		use tauri::Emitter;

		let CloseEmitter = MainWindow.clone();

		MainWindow.on_window_event(move |Event| {
			if let tauri::WindowEvent::CloseRequested { api, .. } = Event {
				api.prevent_close();

				let _ = CloseEmitter.emit("sky://window/close-requested", ());

				dev_log!("window", "[UI] [Window] CloseRequested intercepted; forwarded to webview");
			}
		});
	}

	// -------------------------------------------------------------------------
	// [Backend] [Dirs] Ensure userdata directories exist
	//
	// The cheap, order-sensitive pieces stay synchronous (path-root statics
	// the IPC handlers read, memento hydration the first `storage:getItems`
	// depends on). The 14 `create_dir_all` calls, default-file writes, and
	// the settings.json trust-key injection move to the blocking pool;
	// PostSetup awaits `DirsReady` before its first configuration merge so
	// the trust key is always present when settings.json is read.
	// -------------------------------------------------------------------------
	let DirsReady = {
		let PathResolver = app.path();

		let AppDataDir = PathResolver.app_data_dir().unwrap_or_default();

		let LogDir = PathResolver.app_log_dir().unwrap_or_default();

		let HomeDir = PathResolver.home_dir().unwrap_or_default();

		// Set the canonical userdata base so WindServiceHandlers resolves
		// /User/... paths to the real Tauri app_data_dir (not hardcoded "FIDDEE").
		crate::IPC::WindServiceHandlers::Utilities::UserdataDir::Set::Fn(AppDataDir.to_string_lossy().to_string());

		// Set the real filesystem root for /Static/Application/ path mapping.
		// In dev mode, Tauri serves from ../Sky/Target relative to Mountain.
		// Tauri's resource_dir gives us the frontendDist path.
		// Resolve Sky/Target via Tauri first; fall back to executable-
		// relative bundle and monorepo layouts so raw-binary launches
		// (e.g. running `Target/release/<bin>` directly from a terminal)
		// still resolve `STATIC_APPLICATION_ROOT` correctly. Without this
		// fallback, release binaries launched outside `.app` had an
		// empty static root, causing extension-contributed icons served
		// via `vscode-file://` to 404 (GitLens / Roo / Claude side bar
		// icons missing).
		let SkyTargetDir = PathResolver
			.resource_dir()
			.ok()
			.filter(|P| !P.as_os_str().is_empty() && P.exists())
			.unwrap_or_else(|| {
				let ExeParent = std::env::current_exe()
					.ok()
					.and_then(|Exe| Exe.parent().map(|P| P.to_path_buf()))
					.unwrap_or_default();

				// `.app/Contents/MacOS/<bin>` → `Contents/Resources/`
				let BundleResources = ExeParent.join("../Resources");

				if BundleResources.exists() {
					return BundleResources;
				}

				// Monorepo layout: `Element/Mountain/Target/<profile>/<bin>` →
				// `Element/Sky/Target/`. Used by both debug runs and raw-
				// release launches from inside the repo.
				let RepoSky = ExeParent.join("../../../Sky/Target");

				if RepoSky.exists() {
					return RepoSky;
				}

				// Last resort: alongside the binary. A broken bundle layout
				// then surfaces as visible "asset not found" 404s instead of
				// silent empty-string joins.
				ExeParent
			});

		crate::IPC::WindServiceHandlers::Utilities::ApplicationRoot::Set::Fn(
			SkyTargetDir.to_string_lossy().to_string(),
		);

		dev_log!(
			"lifecycle",
			"[Lifecycle] [Dirs] Static application root: {}",
			SkyTargetDir.display()
		);

		// Set GlobalMementoPath now that we know the real Tauri app data dir
		let GlobalMementoFile = AppDataDir.join("User/globalStorage/global.json");

		{
			let mut Path = app_state.GlobalMementoPath.lock();

			*Path = GlobalMementoFile.clone();
			dev_log!("lifecycle", "[Lifecycle] [Dirs] GlobalMementoPath: {}", Path.display());
		}

		// Boot-time memento hydration: use the crash-safe best-effort loader.
		// A corrupted global.json (partial write during a previous crash, disk
		// corruption, manual edit gone wrong) gets quarantined to a timestamped
		// `.json.corrupted.<ts>` sibling and the in-memory map starts empty
		// rather than panicking the boot path. Workspace memento is loaded on
		// `UpdateWorkspaceMementoPathAndReload` so we only hydrate global here.
		// Kept synchronous: the webview's first `storage:getItems` must see
		// the hydrated map or the workbench loses its persisted state.
		{
			let LoadedGlobal =
				crate::ApplicationState::Internal::Persistence::MementoLoader::LoadInitialMementoFromDisk::Fn(
					&GlobalMementoFile,
				);

			if !LoadedGlobal.is_empty() {
				dev_log!(
					"lifecycle",
					"[Lifecycle] [Memento] Hydrated GlobalMemento ({} keys) from {}",
					LoadedGlobal.len(),
					GlobalMementoFile.display()
				);
			}

			app_state.Configuration.SetGlobalMemento(LoadedGlobal);
		}

		tauri::async_runtime::spawn_blocking(move || {
			// Every directory VS Code may stat or readdir during startup
			let Dirs = [
				// User profile directories
				AppDataDir.join("User"),
				AppDataDir.join("User/globalStorage"),
				AppDataDir.join("User/workspaceStorage"),
				AppDataDir.join("User/workspaceStorage/vscode-chat-images"),
				AppDataDir.join("User/extensions"),
				AppDataDir.join("User/profiles/__default__profile__"),
				AppDataDir.join("User/snippets"),
				AppDataDir.join("User/prompts"),
				AppDataDir.join("User/caches"),
				// Configuration cache
				AppDataDir.join("CachedConfigurations/defaults/__default__profile__-configurationDefaultsOverrides"),
				// Log directories - VS Code stats {logsPath}/window1/output_{timestamp}
				LogDir.join("window1"),
				// System extensions directory - VS Code scans appRoot/../extensions
				// which resolves to /Static/Application/extensions (mapped to Sky Target).
				SkyTargetDir.join("Static/Application/extensions"),
				// Agent directories VS Code probes for (create to avoid stat errors)
				HomeDir.join(".claude/agents"),
				HomeDir.join(".copilot/agents"),
			];

			for Dir in &Dirs {
				if let Err(Error) = std::fs::create_dir_all(Dir) {
					dev_log!(
						"lifecycle",
						"warn: [Lifecycle] [Dirs] Failed to create {}: {}",
						Dir.display(),
						Error
					);
				}
			}

			// Default empty files VS Code reads on startup
			let DefaultFiles:&[(&std::path::Path, &str)] = &[
				(&AppDataDir.join("User/settings.json"), "{}"),
				(&AppDataDir.join("User/keybindings.json"), "[]"),
				(&AppDataDir.join("User/tasks.json"), "{}"),
				(&AppDataDir.join("User/extensions.json"), "[]"),
				(&AppDataDir.join("User/mcp.json"), "{}"),
			];

			for (FilePath, DefaultContent) in DefaultFiles {
				if !FilePath.exists() {
					let _ = std::fs::write(FilePath, DefaultContent);
				}
			}

			// Atom I7: ensure `security.workspace.trust.enabled: false` lives
			// in User/settings.json. Without it, opening the Land repo as a
			// workspace triggers VS Code's workspace-trust gate: built-in
			// extensions whose `location` is inside the picked folder are
			// marked `DisabledByTrustRequirement` (see
			// `extensionEnablementService.ts:549`). Since our built-ins ship
			// under `Element/Sky/Target/Static/Application/extensions/` -
			// which IS inside the repo - any user picking the repo as a
			// workspace hits this filter for every extension. Disabling the
			// trust system wholesale is the correct Land-level policy; we're
			// a personal editor, not a multi-user sandbox. Users can opt
			// back in by flipping this key in their User/settings.json.
			{
				let SettingsPath = AppDataDir.join("User/settings.json");

				let Current = std::fs::read_to_string(&SettingsPath).unwrap_or_else(|_| "{}".to_string());

				if !Current.contains("\"security.workspace.trust.enabled\"") {
					if let Ok(mut Parsed) = serde_json::from_str::<serde_json::Value>(&Current) {
						if !Parsed.is_object() {
							Parsed = serde_json::json!({});
						}

						if let Some(Obj) = Parsed.as_object_mut() {
							Obj.insert("security.workspace.trust.enabled".to_string(), serde_json::Value::Bool(false));
						}

						if let Ok(Serialized) = serde_json::to_string_pretty(&Parsed) {
							let _ = std::fs::write(&SettingsPath, Serialized);

							dev_log!(
								"lifecycle",
								"[Lifecycle] [Dirs] Injected default 'security.workspace.trust.enabled=false' into {}",
								SettingsPath.display()
							);
						}
					}
				}
			}

			dev_log!(
				"lifecycle",
				"[Lifecycle] [Dirs] Userdata directories ensured at {}",
				AppDataDir.display()
			);
		})
	};

	// -------------------------------------------------------------------------
	// [Backend] [Env] Mountain environment
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[Backend] [Env] Creating MountainEnvironment...");

	let Environment = Arc::new(MountainEnvironment::Create(app_handle_for_setup.clone(), app_state.clone()));

	dev_log!("lifecycle", "[Backend] [Env] MountainEnvironment ready.");

	// -------------------------------------------------------------------------
	// [Backend] [Runtime] ApplicationRunTime
	// -------------------------------------------------------------------------
	dev_log!("lifecycle", "[Backend] [Runtime] Creating ApplicationRunTime...");

	let Runtime = Arc::new(ApplicationRunTime::Create(scheduler.clone(), Environment.clone()));

	app_handle_for_setup.manage(Runtime.clone());

	dev_log!("lifecycle", "[Backend] [Runtime] ApplicationRunTime managed.");

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Status Reporter
	// -------------------------------------------------------------------------
	if let Err(e) = StatusReporterRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		dev_log!(
			"lifecycle",
			"error: [Lifecycle] [IPC] Failed to initialize status reporter: {}",
			e
		);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Advanced Features
	// -------------------------------------------------------------------------
	if let Err(e) = AdvancedFeaturesRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		dev_log!(
			"lifecycle",
			"error: [Lifecycle] [IPC] Failed to initialize advanced features: {}",
			e
		);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [IPC] Initialize Wind Advanced Sync
	// -------------------------------------------------------------------------
	if let Err(e) = WindSyncRegisterFn(&app_handle_for_setup, Runtime.clone()) {
		dev_log!(
			"lifecycle",
			"error: [Lifecycle] [IPC] Failed to initialize wind advanced sync: {}",
			e
		);
	}

	// -------------------------------------------------------------------------
	// [Lifecycle] [PostSetup] Async initialization work
	// -------------------------------------------------------------------------
	let PostSetupAppHandle = app_handle_for_setup.clone();

	let PostSetupEnvironment = Environment.clone();

	tauri::async_runtime::spawn(async move {
		dev_log!("lifecycle", "[Lifecycle] [PostSetup] Starting...");

		let PostSetupStart = crate::IPC::DevLog::NowNano::Fn();

		let AppStateForSetup = PostSetupEnvironment.ApplicationState.clone();

		TraceStep!("[Lifecycle] [PostSetup] AppState cloned.");

		// [Dirs] Gate on the background userdata-dir task: the first
		// configuration merge below reads User/settings.json, whose
		// default content and trust-key injection happen in that task.
		let _ = DirsReady.await;

		// [Workspace] [Trust] Desktop app - trust local workspace by default
		AppStateForSetup.Workspace.SetTrustStatus(true);

		// Stage 1: configuration pass 1 ∥ extension scan. Independent -
		// the merge reads settings.json files while the scan walks
		// extension directories into `ScannedExtensions`.
		tokio::join!(
			async {
				// [Config]
				// First-pass merge runs against the (still-populating)
				// `ScannedExtensions` map. User / workspace
				// `settings.json` overrides land here, but extension
				// `contributes.configuration.properties[*].default` keys
				// cannot be relied on yet. Without a second pass after
				// the scan, `getConfiguration('git').get('enabled')`
				// returns undefined, vscode.git's `_activate` takes the
				// `if (!enabled) return;` short-circuit, and the SCM
				// viewlet stays empty even though Cocoon successfully
				// activated the extension. The second pass below repairs
				// this without disturbing the existing initial merge
				// that the rest of bootstrap depends on.
				let ConfigStart = crate::IPC::DevLog::NowNano::Fn();

				let _ = ConfigurationInitializeFn(&PostSetupEnvironment).await;

				crate::otel_span!("lifecycle:config:initialize", ConfigStart);
			},
			async {
				// [Extensions] [ScanPaths]
				let ExtScanStart = crate::IPC::DevLog::NowNano::Fn();

				let _ = ScanPathConfigureFn(&AppStateForSetup);

				// [Extensions] [Scan]
				let _ = ExtensionPopulateFn(PostSetupAppHandle.clone(), &AppStateForSetup).await;

				crate::otel_span!("lifecycle:extensions:scan", ExtScanStart);
			}
		);

		// Stage 2: configuration re-merge ∥ Vine gRPC server start. The
		// re-merge needs the finished extension scan (stage 1); Vine only
		// binds its listeners and needs neither configuration nor scan.
		tokio::join!(
			async {
				// [Config] [Re-merge] - now that ScannedExtensions is populated,
				// run the merge a second time so `collect_default_configurations`
				// can walk extension manifests and seed `git.enabled = true`,
				// `git.path = null`, `git.autoRepositoryDetection = true`, plus
				// every other `contributes.configuration.properties[*].default`
				// the 113 scanned extensions declare. The first-pass merge logged
				// "0 top-level keys"; this pass should log a much larger count.
				// User / workspace overrides applied during the first pass are
				// preserved because the merge order is Default → User → Workspace
				// and the cached User/Workspace JSON files are re-read each call.
				let ConfigRemergeStart = crate::IPC::DevLog::NowNano::Fn();

				let _ = ConfigurationInitializeFn(&PostSetupEnvironment).await;

				crate::otel_span!("lifecycle:config:remerge-after-extension-scan", ConfigRemergeStart);
			},
			async {
				// [Vine] [gRPC]
				let VineStart = crate::IPC::DevLog::NowNano::Fn();

				let _ = VineStartFn(
					PostSetupAppHandle.clone(),
					"127.0.0.1:50051".to_string(),
					"127.0.0.1:50052".to_string(),
				)
				.await;

				crate::otel_span!("lifecycle:vine:start", VineStart);
			}
		);

		// Stage 3: Cocoon ∥ Air. Both sidecars connect back to the Vine
		// pool started in stage 2; Cocoon additionally consumes the
		// finished extension scan (stage 1) for its init payload. They
		// have no dependency on each other.
		tokio::join!(
			async {
				// [Cocoon] [Sidecar] - skipped when Disable=true so the
				// workbench loads without an extension host. Useful for
				// bisecting whether typing-input regressions originate in
				// Cocoon's gRPC handlers or upstream / Tauri / WKWebView.
				if IsLandDisabled() {
					dev_log!(
						"cocoon",
						"[Cocoon] [Start] Disable=true: Cocoon spawn SKIPPED (workbench will run without extensions)"
					);
				} else {
					let CocoonStart = crate::IPC::DevLog::NowNano::Fn();

					let _ = CocoonStartFn(&PostSetupAppHandle, &PostSetupEnvironment).await;

					crate::otel_span!("lifecycle:cocoon:start", CocoonStart);
				}
			},
			async {
				// [Air] [Sidecar] - daemon for updates / downloads / signing /
				// indexing / system monitoring. Spawn parallel to Cocoon; both
				// are sidecars in the Vine pool. AirStart returns Ok(()) even
				// on spawn failure (graceful degradation - workbench works
				// without Air, just without those background capabilities).
				// Skipped under `Disable=true` for parity with Cocoon.
				if IsLandDisabled() {
					dev_log!("grpc", "[Air] [Start] Disable=true: Air spawn SKIPPED");
				} else {
					let AirStartT0 = crate::IPC::DevLog::NowNano::Fn();

					let _ = AirStartFn(&PostSetupAppHandle, &PostSetupEnvironment).await;

					crate::otel_span!("lifecycle:air:start", AirStartT0);
				}
			}
		);

		// [Lifecycle] [Phase] Advance Starting → Ready now that the gRPC
		// server + Cocoon sidecar + extension scan have all finished. Wind's
		// `TauriChannel("lifecycle").listen("onDidChangePhase")` subscribers
		// fire so long-running services can start pulling.
		AppStateForSetup.Feature.Lifecycle.AdvanceAndBroadcast(2, &PostSetupAppHandle);

		// Schedule a background transition to Restored (3), then Eventually
		// (4). Sky/Wind are the authoritative signal - they call
		// `lifecycle:advancePhase` over Tauri IPC when the workbench is
		// truly interactive (`Restored`) and when late-binding extensions
		// should stop blocking (`Eventually`). `AdvanceAndBroadcast`
		// rejects backwards/same-phase advances (LifecyclePhaseState.rs:53),
		// so the timers below are pure fallbacks: if Sky has already driven
		// the phase, these become no-ops and log nothing visible.
		//
		// The windows are deliberately generous - a debug-electron cold
		// boot with 98 extensions has been observed to finish its
		// `$activateByEvent("*")` burst at ~3.5 s on an M4 mini and
		// noticeably later on older hardware. The previous 2 s / 5 s
		// timings ran the risk of flipping Restored while the burst was
		// still in flight, which prematurely unblocked services gated on
		// "the editor is interactive". 8 s / 15 s keeps a safety margin
		// without visibly delaying late-binding extensions that legitimately
		// need Eventually to fire.
		let LifecycleStateClone = AppStateForSetup.Feature.Lifecycle.clone();

		let AppHandleForPhase = PostSetupAppHandle.clone();

		tauri::async_runtime::spawn(async move {
			tokio::time::sleep(tokio::time::Duration::from_millis(8_000)).await;

			if LifecycleStateClone.GetPhase() < 3 {
				dev_log!(
					"lifecycle",
					"[Lifecycle] [Fallback] Sky did not advance to Restored within 8s; Mountain auto-advancing \
					 (current phase={})",
					LifecycleStateClone.GetPhase()
				);

				LifecycleStateClone.AdvanceAndBroadcast(3, &AppHandleForPhase);
			}

			tokio::time::sleep(tokio::time::Duration::from_millis(15_000)).await;

			if LifecycleStateClone.GetPhase() < 4 {
				dev_log!(
					"lifecycle",
					"[Lifecycle] [Fallback] Sky did not advance to Eventually within 23s total; Mountain \
					 auto-advancing (current phase={})",
					LifecycleStateClone.GetPhase()
				);

				LifecycleStateClone.AdvanceAndBroadcast(4, &AppHandleForPhase);
			}
		});

		// Hidden-until-ready safety timer: `WindowBuild.rs` creates the main
		// window with `.visible(false)` and the `lifecycle:advancePhase(3)`
		// handler reveals it once Sky reports the workbench DOM is attached.
		// If Sky crashes before phase 3 reaches Mountain, the window would
		// stay invisible forever. Force-reveal after 3 s so the user always
		// sees SOMETHING even on a completely broken Sky. 3 s matches the
		// observed p95 of `[Lifecycle] [Phase] Advance Ready` on a cold
		// M-series boot, so the timer rarely fires on a healthy path.
		let AppHandleForEmergencyShow = PostSetupAppHandle.clone();

		tauri::async_runtime::spawn(async move {
			tokio::time::sleep(tokio::time::Duration::from_millis(3_000)).await;

			if let Some(MainWindow) = AppHandleForEmergencyShow.get_webview_window("main") {
				if let Ok(false) = MainWindow.is_visible() {
					dev_log!(
						"lifecycle",
						"warn: [Lifecycle] [Fallback] main window hidden at +3s; force-revealing to avoid an \
						 invisible-window lockup (Sky never reached phase 3)"
					);

					let _ = MainWindow.show();

					let _ = MainWindow.set_focus();
				}
			}
		});

		crate::otel_span!("lifecycle:postsetup:complete", PostSetupStart);

		dev_log!("lifecycle", "[Lifecycle] [PostSetup] Complete. System ready.");
	});

	Ok(())
}
