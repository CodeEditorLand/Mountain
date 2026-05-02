//! # Cocoon Management
//!
//! This module provides comprehensive lifecycle management for the Cocoon
//! sidecar process, which serves as the VS Code extension host within the
//! Mountain editor.
//!
//! ## Overview
//!
//! Cocoon is a Node.js-based process that provides compatibility with VS Code
//! extensions. This module handles:
//!
//! - **Process Spawning**: Launching Node.js with the Cocoon bootstrap script
//! - **Environment Configuration**: Setting up environment variables for IPC
//!   and logging
//! - **Communication Setup**: Establishing gRPC/Vine connections on port 50052
//! - **Health Monitoring**: Tracking process state and handling failures
//! - **Lifecycle Management**: Graceful shutdown and restart capabilities
//! - **IO Redirection**: Capturing stdout/stderr for logging and debugging
//!
//! ## Process Communication
//!
//! The Cocoon process communicates via:
//! - gRPC on port 50052 (configured via MOUNTAIN_GRPC_PORT/COCOON_GRPC_PORT)
//! - Vine protocol for cross-process messaging
//! - Standard streams for logging (VSCODE_PIPE_LOGGING)
//!
//! ## Dependencies
//!
//! - `scripts/cocoon/bootstrap-fork.js`: Bootstrap script for launching Cocoon
//! - Node.js runtime: Required for executing Cocoon
//! - Vine gRPC server: Must be running on port 50051 for handshake
//!
//! ## Error Handling
//!
//! The module provides graceful degradation:
//! - If the bootstrap script is missing, returns `FileSystemNotFound` error
//! - If Node.js cannot be spawned, returns `IPCError`
//! - If gRPC connection fails, returns `IPCError` with context
//!
//! # Module Contents
//!
//! - [`InitializeCocoon`]: Main entry point for Cocoon initialization
//! - `LaunchAndManageCocoonSideCar`: Process spawning and lifecycle
//! management
//!
//! ## Example
//!
//! ```rust,no_run
//! use crate::Source::ProcessManagement::CocoonManagement::InitializeCocoon;
//!
//! // Initialize Cocoon with application handle and environment
//! match InitializeCocoon(&app_handle, &environment).await {
//! 	Ok(()) => println!("Cocoon initialized successfully"),
//! 	Err(e) => eprintln!("Cocoon initialization failed: {:?}", e),
//! }
//! ```

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use CommonLibrary::Error::CommonError::CommonError;
use tauri::{
	AppHandle,
	Manager,
	Wry,
	path::{BaseDirectory, PathResolver},
};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::{Child, Command},
	sync::Mutex,
	time::sleep,
};

use super::{InitializationData, NodeResolver};
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::Common::HealthStatus::{HealthIssue, HealthMonitor},
	ProcessManagement::ExtractDevTag::Fn as ExtractDevTag,
	Vine,
	dev_log,
};

/// Configuration constants for Cocoon process management
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";
const COCOON_GRPC_PORT:u16 = 50052;
const MOUNTAIN_GRPC_PORT:u16 = 50051;
const BOOTSTRAP_SCRIPT_PATH:&str = "scripts/cocoon/bootstrap-fork.js";

/// Exponential-backoff retry parameters for the Mountain → Cocoon gRPC
/// handshake. Replaces the previous "20 × 1000 ms fixed poll" which
/// under-probed the common race (Cocoon's stage2 binds the port at
/// ~200 ms so attempts 1-2 fail and we sat idle through 18 more whole-
/// second sleeps) and over-waited the real failure (when Cocoon is
/// genuinely dead, we wasted 20 s before reporting).
///
/// Policy: start at 50 ms, double each attempt up to a 2 s ceiling,
/// with a hard 20 s total-budget. Under healthy spawn timing (Cocoon
/// up at 150-600 ms) this converges on attempts 3-5 in <~400 ms total;
/// under a genuinely dead Cocoon the loop abandons at the budget.
const GRPC_CONNECT_INITIAL_MS:u64 = 50;
const GRPC_CONNECT_MAX_DELAY_MS:u64 = 2_000;
const GRPC_CONNECT_BUDGET_MS:u64 = 20_000;

/// Relative path from the resolved Cocoon package root to the bundled
/// entry module. Used by the pre-flight guard below to fail fast with
/// an actionable error when the bundle is missing (esbuild failure,
/// partial rm -rf, freshly cloned checkout without `pnpm run
/// prepublishOnly`, etc.) instead of spawning Node into a dying
/// require() chain.
const COCOON_BUNDLE_PROBE:&str = "../Cocoon/Target/Bootstrap/Implementation/CocoonMain.js";
const HANDSHAKE_TIMEOUT_MS:u64 = 60000;
const HEALTH_CHECK_INTERVAL_SECONDS:u64 = 5;
#[allow(dead_code)]
const MAX_RESTART_ATTEMPTS:u32 = 3;
#[allow(dead_code)]
const RESTART_WINDOW_SECONDS:u64 = 300;

/// Global state for tracking Cocoon process lifecycle
#[allow(dead_code)]
struct CocoonProcessState {
	ChildProcess:Option<Child>,
	IsRunning:bool,
	StartTime:Option<tokio::time::Instant>,
	RestartCount:u32,
	LastRestartTime:Option<tokio::time::Instant>,
}

impl Default for CocoonProcessState {
	fn default() -> Self {
		Self {
			ChildProcess:None,
			IsRunning:false,
			StartTime:None,
			RestartCount:0,
			LastRestartTime:None,
		}
	}
}

// Global state for Cocoon process management
lazy_static::lazy_static! {
	static ref COCOON_STATE: Arc<Mutex<CocoonProcessState>> =
		Arc::new(Mutex::new(CocoonProcessState::default()));

	static ref COCOON_HEALTH: Arc<Mutex<HealthMonitor>> =
		Arc::new(Mutex::new(HealthMonitor::new()));
}

/// Last-known PID of the Cocoon child process. Mirrored here so callers can
/// read it without taking the async `COCOON_STATE` mutex (e.g. from IPC
/// handlers such as `extensionHostStarter:start`). Set after spawn and
/// cleared on shutdown. `0` means "not running".
static COCOON_PID:std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Return the Cocoon child process's OS PID, or `None` if Cocoon has not
/// been spawned (or has exited).
pub fn GetCocoonPid() -> Option<u32> {
	match COCOON_PID.load(std::sync::atomic::Ordering::Relaxed) {
		0 => None,
		Pid => Some(Pid),
	}
}

/// The main entry point for initializing the Cocoon sidecar process manager.
///
/// This orchestrates the complete initialization sequence including:
/// - Validating feature flags and dependencies
/// - Launching the Cocoon process with proper configuration
/// - Establishing gRPC communication
/// - Performing the initialization handshake
/// - Setting up process health monitoring
///
/// # Arguments
///
/// * `ApplicationHandle` - Tauri application handle for path resolution
/// * `Environment` - Mountain environment containing application state and
///   services
///
/// # Returns
///
/// * `Ok(())` - Cocoon initialized successfully and ready to accept extension
///   requests
/// * `Err(CommonError)` - Initialization failed with detailed error context
///
/// # Errors
///
/// - `FileSystemNotFound`: Bootstrap script not found
/// - `IPCError`: Failed to spawn process or establish gRPC connection
///
/// # Example
///
/// ```rust,no_run
/// use crate::Source::ProcessManagement::CocoonManagement::InitializeCocoon;
///
/// InitializeCocoon(&app_handle, &environment).await?;
/// ```
pub async fn InitializeCocoon(
	ApplicationHandle:&AppHandle,
	Environment:&Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	dev_log!("cocoon", "[CocoonManagement] Initializing Cocoon sidecar manager...");

	// Atom N1: `debug-mountain-only` / `release-mountain-only` profiles set
	// Spawn=false so Mountain boots without the extension host.
	// Extension-related IPC returns the empty-state envelope; the workbench
	// loads but no extension activates. Useful for integration tests that
	// exercise Mountain in isolation and for the smallest shippable surface.
	if matches!(std::env::var("Spawn").as_deref(), Ok("0") | Ok("false")) {
		dev_log!("cocoon", "[CocoonManagement] Skipping spawn (Spawn=false)");
		return Ok(());
	}

	#[cfg(feature = "ExtensionHostCocoon")]
	{
		LaunchAndManageCocoonSideCar(ApplicationHandle.clone(), Environment.clone()).await
	}

	#[cfg(not(feature = "ExtensionHostCocoon"))]
	{
		dev_log!(
			"cocoon",
			"[CocoonManagement] 'ExtensionHostCocoon' feature is disabled. Cocoon will not be launched."
		);
		Ok(())
	}
}

/// Spawns the Cocoon process, manages its communication channels, and performs
/// the complete initialization handshake sequence.
///
/// This function implements the complete Cocoon lifecycle:
/// 1. Validates bootstrap script availability
/// 2. Constructs environment variables for IPC and logging
/// 3. Spawns Node.js process with proper IO redirection
/// 4. Captures stdout/stderr for logging
/// 5. Waits for gRPC server to be ready
/// 6. Establishes Vine connection
/// 7. Sends initialization payload and validates response
///
/// # Arguments
///
/// * `ApplicationHandle` - Tauri application handle for resolving resource
///   paths
/// * `Environment` - Mountain environment containing application state
///
/// # Returns
///
/// * `Ok(())` - Cocoon process spawned, connected, and initialized successfully
/// * `Err(CommonError)` - Any failure during the initialization sequence
///
/// # Errors
///
/// - `FileSystemNotFound`: Bootstrap script not found in resources
/// - `IPCError`: Failed to spawn process, connect gRPC, or complete handshake
///
/// # Lifecycle
///
/// The process runs as a background task with IO redirection for logging.
/// Process failures are logged but not automatically restarted (callers should
/// implement restart strategies based on their requirements).
async fn LaunchAndManageCocoonSideCar(
	ApplicationHandle:AppHandle,
	Environment:Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	let SideCarIdentifier = COCOON_SIDE_CAR_IDENTIFIER.to_string();
	let path_resolver:PathResolver<Wry> = ApplicationHandle.path().clone();

	// Resolve bootstrap script path.
	// 1) Try Tauri bundled resources (production builds).
	// 2) Fallback: resolve relative to the executable (dev builds). Dev layout:
	//    Target/debug/binary → ../../scripts/cocoon/bootstrap-fork.js
	let ScriptPath = path_resolver
		.resolve(BOOTSTRAP_SCRIPT_PATH, BaseDirectory::Resource)
		.ok()
		.filter(|P| P.exists())
		.or_else(|| {
			std::env::current_exe().ok().and_then(|Exe| {
				let MountainRoot = Exe.parent()?.parent()?.parent()?;
				let Candidate = MountainRoot.join(BOOTSTRAP_SCRIPT_PATH);
				if Candidate.exists() { Some(Candidate) } else { None }
			})
		})
		.ok_or_else(|| {
			CommonError::FileSystemNotFound(
				format!(
					"Cocoon bootstrap script '{}' not found in resources or relative to executable",
					BOOTSTRAP_SCRIPT_PATH
				)
				.into(),
			)
		})?;

	dev_log!(
		"cocoon",
		"[CocoonManagement] Found bootstrap script at: {}",
		ScriptPath.display()
	);
	crate::dev_log!("cocoon", "bootstrap script: {}", ScriptPath.display());

	// Pre-flight: Cocoon's bundle must exist or the spawned Node will
	// die silently on the first `import()` and we'll sit through 20+
	// seconds of `attempt N/M` retries with no diagnostic.
	//
	// bootstrap-fork.js is in `Mountain/scripts/cocoon/`. The Cocoon
	// bundle is at `Cocoon/Target/Bootstrap/Implementation/CocoonMain.js`
	// relative to the repo root. Compose the probe path by walking up
	// from the bootstrap script to the `Element/` root, then descending.
	if let Some(BootstrapDirectory) = ScriptPath.parent() {
		let ProbePath = BootstrapDirectory.join("../..").join(COCOON_BUNDLE_PROBE);
		if !ProbePath.exists() {
			return Err(CommonError::IPCError {
				Description:format!(
					"Cocoon bundle is missing at {}. Run `pnpm run prepublishOnly --filter=@codeeditorland/cocoon` \
					 (or the full `./Maintain/Debug/Build.sh --profile debug-electron`) before launching - node will \
					 fail to import without it and Mountain will fall into degraded mode with zero extensions \
					 available. Root cause is typically an esbuild failure in an upstream Cocoon source file or a \
					 stale `rm -rf Element/Cocoon/Target` without a rebuild.",
					ProbePath.display()
				),
			});
		}
		dev_log!("cocoon", "[CocoonManagement] pre-flight OK: bundle at {}", ProbePath.display());
	}

	// Atom I6: zombie-Cocoon sweep. If a prior Mountain exited without
	// killing its child (segfault, SIGKILL, debugger detach, …), the stale
	// node process keeps port COCOON_GRPC_PORT bound. The new Mountain's
	// VineClient then "successfully connects" to the zombie while the
	// freshly-spawned Cocoon fails to bind with EADDRINUSE, and the whole
	// extension host enters degraded mode with zero extensions visible.
	//
	// Probe the port. If it answers, find the owning PID via `lsof -t -i
	// :<port>` and SIGTERM → 500ms wait → SIGKILL. Then proceed as normal.
	SweepStaleCocoon(COCOON_GRPC_PORT);

	// Atom N1: resolve Node binary via NodeResolver (shipped → version
	// managers → homebrew → PATH). Logs the pick + source for forensics.
	// Overridable via `Pick=/absolute/path/to/node`.
	let ResolvedNodeBinary = NodeResolver::ResolveNodeBinary::Fn(&ApplicationHandle);

	// Build Node.js command with comprehensive environment configuration
	let mut NodeCommand = Command::new(&ResolvedNodeBinary.Path);

	let mut EnvironmentVariables = HashMap::new();

	// VS Code protocol environment variables for extension host compatibility
	EnvironmentVariables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());

	// gRPC port configuration for Vine communication
	EnvironmentVariables.insert("MOUNTAIN_GRPC_PORT".to_string(), MOUNTAIN_GRPC_PORT.to_string());
	EnvironmentVariables.insert("COCOON_GRPC_PORT".to_string(), COCOON_GRPC_PORT.to_string());

	// Preserve PATH so `node` resolves. env_clear() was stripping it.
	if let Ok(Path) = std::env::var("PATH") {
		EnvironmentVariables.insert("PATH".to_string(), Path);
	}
	if let Ok(Home) = std::env::var("HOME") {
		EnvironmentVariables.insert("HOME".to_string(), Home);
	}

	// Atom I5: forward every Product*, Tier*, Network* env var from
	// .env.Land into the Cocoon subprocess. Cocoon's InitData.ts +
	// ExtensionHostHandler.ts read these at startup for version,
	// identity, and port configuration. Without this forwarding, the
	// whitelist above drops them and Cocoon falls back to defaults,
	// defeating the single-source-of-truth design.
	//
	// PascalCase single-word vars: covers `.env.Land.PostHog` (Authorize,
	// Beam, Report, Brand, Replay, Ask, Throttle, Buffer, Batch, Cap),
	// `.env.Land.Node` (Pick, Require), `.env.Land.Extensions` (Lodge,
	// Extend, Probe, Ship, Wire, Install, Mute, Skip), and the
	// kernel / Cocoon-spawn / preload gating flags (Spawn, Render).
	// Each name is a single PascalCase action verb - no LAND_ prefix.
	// Previously only Product/Tier/Network were forwarded and the
	// PostHog bridge fell back to the empty-string default; the
	// AllowList below now enumerates every Land-introduced env var by
	// name so Cocoon sees the same values Mountain reads.
	const LandEnvAllowList:&[&str] = &[
		"Authorize",
		"Beam",
		"Report",
		"Brand",
		"Replay",
		"Ask",
		"Throttle",
		"Buffer",
		"Batch",
		"Cap",
		"Pick",
		"Require",
		"Lodge",
		"Extend",
		"Probe",
		"Ship",
		"Wire",
		"Install",
		"Mute",
		"Skip",
		"Spawn",
		"Render",
		"Walk",
		"Trace",
		"Record",
		"Profile",
		"Diagnose",
		"Resolve",
		"Open",
		"Warn",
		"Catch",
		"Source",
		"Track",
		"Defer",
		"Boot",
		"Pack",
	];
	for (Key, Value) in std::env::vars() {
		if Key.starts_with("Product")
			|| Key.starts_with("Tier")
			|| Key.starts_with("Network")
			|| LandEnvAllowList.contains(&Key.as_str())
		{
			EnvironmentVariables.insert(Key, Value);
		}
	}

	// Atom I11: forward NODE_ENV / TAURI_ENV_DEBUG (Trace is
	// already covered by the `LAND_` prefix sweep above). Without this,
	// env_clear() leaves Cocoon seeing NodeEnv="production" /
	// TauriDebug=false even on the debug-electron profile - silently
	// disabling dev-only logging and debug-only diagnostics in Cocoon.
	for Key in ["NODE_ENV", "TAURI_ENV_DEBUG"] {
		if let Ok(Value) = std::env::var(Key) {
			EnvironmentVariables.insert(Key.to_string(), Value);
		}
	}

	NodeCommand
		.arg(&ScriptPath)
		.env_clear()
		.envs(EnvironmentVariables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// Spawn the process with error handling
	let mut ChildProcess = NodeCommand.spawn().map_err(|Error| {
		CommonError::IPCError {
			Description:format!(
				"Failed to spawn Cocoon with node={} (source={}): {}. Override with Pick=/absolute/path or install \
				 Node.js.",
				ResolvedNodeBinary.Path.display(),
				ResolvedNodeBinary.Source.AsLabel(),
				Error
			),
		}
	})?;

	let ProcessId = ChildProcess.id().unwrap_or(0);
	COCOON_PID.store(ProcessId, std::sync::atomic::Ordering::Relaxed);
	dev_log!("cocoon", "[CocoonManagement] Cocoon process spawned [PID: {}]", ProcessId);
	crate::dev_log!("cocoon", "spawned PID={}", ProcessId);

	// Capture stdout for trace logging. Two disposition classes:
	//
	// 1. Tagged lines produced by `Cocoon/Source/Services/DevLog.ts::
	//    CocoonDevLog(Tag, Message)` arrive prefixed with `[DEV:<UPPER_TAG>]
	//    <body>`. Re-emit under the matching Mountain tag (lowercased) so
	//    `Trace=bootstrap-stage` on Mountain's side surfaces Cocoon's
	//    `bootstrap-stage` lines without forcing the user to also enable the broad
	//    `cocoon` tag.
	//
	// 2. Plain stdout (console.log, uncaught trace, etc.) stays under the `cocoon`
	//    tag so it's silent unless explicitly requested.
	if let Some(stdout) = ChildProcess.stdout.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stdout);
			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				if let Some(ForwardedTag) = ExtractDevTag(&Line) {
					// dev_log! macro requires a static string, so match on
					// the known tag set and fall through to raw 'cocoon'
					// for anything else. Keep the arms in sync with
					// `CocoonDevLog` call sites.
					match ForwardedTag.as_str() {
						"bootstrap-stage" => dev_log!("bootstrap-stage", "[Cocoon stdout] {}", Line),
						"ext-activate" => dev_log!("ext-activate", "[Cocoon stdout] {}", Line),
						"config-prime" => dev_log!("config-prime", "[Cocoon stdout] {}", Line),
						"breaker" => dev_log!("breaker", "[Cocoon stdout] {}", Line),
						_ => dev_log!("cocoon", "[Cocoon stdout] {}", Line),
					}
				} else {
					dev_log!("cocoon", "[Cocoon stdout] {}", Line);
				}
			}
		});
	}

	// Capture stderr for warn-level logging.
	//
	// Node and macOS tooling write a stream of informational-only noise
	// to stderr that is indistinguishable from fatal errors at the line
	// level. Downgrade these to the verbose `cocoon-stderr-verbose` tag
	// (silent under `Trace=short`) so the main cocoon channel only
	// carries actionable Node errors:
	//
	// - `: is already signed` / `: replacing existing signature` - macOS codesign
	//   informational output when Cocoon re-signs a just-rebuilt extension binary.
	//   Not an error.
	// - `DeprecationWarning:` / `(node:...) [DEP0...]` - Node deprecation warnings
	//   from VS Code's upstream dependencies (punycode, url.parse, Buffer()).
	//   Fixable only in upstream, not in Land.
	// - `Use \`node --trace-deprecation\` to show where the warning was created` -
	//   follow-up to the DEP line above.
	// - `EntryNotFound (FileSystemError):` + follow-up stack frames - extensions
	//   (svelte, copilot, etc.) probe paths that may not exist and let the
	//   rejection bubble up. Node's unhandled rejection printer splits the stack
	//   across stderr lines. The classifier enters a stateful "suppress follow-up
	//   stack frames" mode after the first EntryNotFound line and exits on a
	//   non-frame line.
	if let Some(stderr) = ChildProcess.stderr.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stderr);
			let mut Lines = Reader.lines();
			let mut SuppressStackFrames = false;

			while let Ok(Some(Line)) = Lines.next_line().await {
				let Trimmed = Line.trim_start();
				let IsStackFrame = Trimmed.starts_with("at ")
					|| Trimmed.starts_with("code: '")
					|| Trimmed == "}"
					|| Trimmed.is_empty();
				if SuppressStackFrames && IsStackFrame {
					dev_log!("cocoon-stderr-verbose", "[Cocoon stderr] {}", Line);
					continue;
				}
				// Exited the suppression window. Reset and classify
				// this line normally.
				SuppressStackFrames = false;

				let IsBenignSingleLine = Line.contains(": is already signed")
					|| Line.contains(": replacing existing signature")
					|| Line.contains("DeprecationWarning:")
					|| Line.contains("--trace-deprecation")
					|| Line.contains("--trace-warnings");
				let IsBenignStackHead = Line.contains("EntryNotFound (FileSystemError):")
					|| Line.contains("FileNotFound (FileSystemError):")
					|| Line.contains("[LandFix:UnhandledRejection]")
					|| Line.starts_with("[Patcher] unhandledRejection:")
					|| Line.starts_with("[Patcher] uncaughtException:");
				if IsBenignStackHead {
					SuppressStackFrames = true;
				}
				if IsBenignSingleLine || IsBenignStackHead {
					dev_log!("cocoon-stderr-verbose", "[Cocoon stderr] {}", Line);
				} else {
					dev_log!("cocoon", "warn: [Cocoon stderr] {}", Line);
				}
			}
		});
	}

	// Establish Vine connection to Cocoon with exponential-backoff
	// retry + child-exit detection.
	//
	// Prior policy was 20 × 1000 ms fixed poll. Under healthy timing
	// (Cocoon binds at 150-600 ms) that wasted ~400 ms of idle time
	// every boot; under a genuinely dead Cocoon (import error, killed
	// process, stale bundle) it burned 20 full seconds before giving
	// up with a generic "is Cocoon running?" hint.
	//
	// New policy:
	//   - Initial 50 ms sleep, doubled per attempt up to a 2 s ceiling.
	//   - Hard 20 s total-budget (unchanged) so the overall failure ceiling doesn't
	//     regress for pathological slow-boot hardware.
	//   - Before each sleep, poll `ChildProcess.try_wait()`: if Node has exited,
	//     abandon the loop immediately with the exit status embedded in the error -
	//     no point retrying against a dead process, and the exit code usually
	//     reveals the import failure (1 = unhandled exception, 13 = invalid
	//     module).
	let GRPCAddress = format!("127.0.0.1:{}", COCOON_GRPC_PORT);
	dev_log!(
		"cocoon",
		"[CocoonManagement] Connecting to Cocoon gRPC at {} (exponential backoff, budget={}ms)...",
		GRPCAddress,
		GRPC_CONNECT_BUDGET_MS
	);

	let ConnectStart = tokio::time::Instant::now();
	let mut CurrentDelayMs:u64 = GRPC_CONNECT_INITIAL_MS;
	let mut ConnectAttempt = 0u32;

	loop {
		ConnectAttempt += 1;
		crate::dev_log!(
			"grpc",
			"connecting to Cocoon at {} (attempt {}, elapsed={}ms)",
			GRPCAddress,
			ConnectAttempt,
			ConnectStart.elapsed().as_millis()
		);

		match Vine::Client::ConnectToSideCar::Fn(SideCarIdentifier.clone(), GRPCAddress.clone()).await {
			Ok(()) => {
				crate::dev_log!(
					"grpc",
					"connected to Cocoon on attempt {} (elapsed={}ms)",
					ConnectAttempt,
					ConnectStart.elapsed().as_millis()
				);
				break;
			},
			Err(Error) => {
				// Check if the Node child has already died. If yes,
				// there is no point waiting any longer - report the
				// real exit status so the dev log points at the real
				// failure (import error, crash, oom kill) instead of
				// the abstract "connect refused" message.
				match ChildProcess.try_wait() {
					Ok(Some(ExitStatus)) => {
						let ExitCode = ExitStatus.code().unwrap_or(-1);
						crate::dev_log!(
							"grpc",
							"attempt {} aborted: Cocoon Node process exited with code={} after {}ms - stderr above \
							 (if any) explains why",
							ConnectAttempt,
							ExitCode,
							ConnectStart.elapsed().as_millis()
						);
						return Err(CommonError::IPCError {
							Description:format!(
								"Cocoon spawned but exited with code {} before Mountain could connect. See \
								 `[DEV:COCOON] warn: [Cocoon stderr] …` lines above for the Node-side error - \
								 typically a missing bundle (\"Cannot find module …\") or an ESM/CJS import drift \
								 after a partial build.",
								ExitCode
							),
						});
					},
					Ok(None) => { /* still running, keep trying */ },
					Err(WaitErr) => {
						// try_wait() itself failed; this is rare
						// (would imply a kernel-level issue). Surface
						// it but keep trying - the dial may still
						// succeed on the next attempt.
						crate::dev_log!("grpc", "warn: try_wait on Cocoon child failed: {} (continuing)", WaitErr);
					},
				}

				let Elapsed = ConnectStart.elapsed().as_millis() as u64;
				if Elapsed >= GRPC_CONNECT_BUDGET_MS {
					crate::dev_log!(
						"grpc",
						"attempt {} timed out (budget {}ms exhausted): {}",
						ConnectAttempt,
						GRPC_CONNECT_BUDGET_MS,
						Error
					);
					return Err(CommonError::IPCError {
						Description:format!(
							"Failed to connect to Cocoon gRPC at {} after {} attempts over {}ms: {} (is Cocoon \
							 running? check `[DEV:COCOON]` log lines for stderr, or re-run with the debug-electron \
							 build profile if the bundle is stale)",
							GRPCAddress, ConnectAttempt, GRPC_CONNECT_BUDGET_MS, Error
						),
					});
				}

				crate::dev_log!(
					"grpc",
					"attempt {} pending (Cocoon still booting): {}, backing off {}ms",
					ConnectAttempt,
					Error,
					CurrentDelayMs
				);

				sleep(Duration::from_millis(CurrentDelayMs)).await;
				// Exponential ramp with a 2 s ceiling. Doubling keeps
				// the common case fast (4 attempts cover the first
				// 750 ms) and the cold-boot case bounded.
				CurrentDelayMs = (CurrentDelayMs * 2).min(GRPC_CONNECT_MAX_DELAY_MS);
			},
		}
	}

	dev_log!(
		"cocoon",
		"[CocoonManagement] Connected to Cocoon. Sending initialization data..."
	);

	// Brief delay to ensure Cocoon's gRPC service handlers are fully registered
	// after bindAsync resolves (race condition on fast connections like attempt 1)
	sleep(Duration::from_millis(200)).await;

	// Construct initialization payload
	let MainInitializationData = InitializationData::ConstructExtensionHostInitializationData(&Environment)
		.await
		.map_err(|Error| {
			CommonError::IPCError { Description:format!("Failed to construct initialization data: {}", Error) }
		})?;

	// Send initialization request with timeout
	let Response = Vine::Client::SendRequest::Fn(
		&SideCarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		HANDSHAKE_TIMEOUT_MS,
	)
	.await
	.map_err(|Error| {
		CommonError::IPCError {
			Description:format!("Failed to send initialization request to Cocoon: {}", Error),
		}
	})?;

	// Validate handshake response
	match Response.as_str() {
		Some("initialized") => {
			dev_log!(
				"cocoon",
				"[CocoonManagement] Cocoon handshake complete. Extension host is ready."
			);
		},
		Some(other) => {
			return Err(CommonError::IPCError {
				Description:format!("Cocoon initialization failed with unexpected response: {}", other),
			});
		},
		None => {
			return Err(CommonError::IPCError {
				Description:"Cocoon initialization failed: no response received".to_string(),
			});
		},
	}

	// Trigger startup extension activation. Cocoon is fully reactive -
	// it won't activate any extensions until Mountain tells it to.
	// Fire-and-forget: don't block on activation, and don't fail init if it errors.
	//
	// Stock VS Code fires a cascade of activation events at boot:
	//   1. `*` - unconditional "activate anything that contributes *"
	//   2. `onStartupFinished` - queued extensions whose start may be deferred
	//      until after the first frame renders
	//   3. `workspaceContains:<pattern>` for each pattern any extension
	//      contributes, fired per matching workspace folder
	//
	// Previously only `*` fired, which meant a large class of extensions
	// that gate on `workspaceContains:package.json`, `onStartupFinished`,
	// or similar events never activated without user interaction. The
	// added bursts below bring startup coverage in line with stock.
	let SideCarId = SideCarIdentifier.clone();
	let EnvironmentForActivation = Environment.clone();
	tokio::spawn(async move {
		// Small delay to let Cocoon finish processing the init response
		sleep(Duration::from_millis(500)).await;

		crate::dev_log!("exthost", "Sending $activateByEvent(\"*\") to Cocoon");

		if let Err(Error) = Vine::Client::SendRequest::Fn(
			&SideCarId,
			"$activateByEvent".to_string(),
			serde_json::json!({ "activationEvent": "*" }),
			30_000,
		)
		.await
		{
			dev_log!("cocoon", "warn: [CocoonManagement] $activateByEvent(\"*\") failed: {}", Error);
			return;
		}
		dev_log!("cocoon", "[CocoonManagement] Startup extensions activation (*) triggered");

		// Phase 2: workspaceContains: events. Iterate the scanned
		// extension registry, collect every pattern contributed via the
		// `workspaceContains:<pattern>` activation event, and fire the
		// event if at least one workspace folder contains a path
		// matching the pattern. Patterns are treated as filename globs
		// relative to any workspace folder root; matching is done with
		// a lightweight walk bounded by depth 3 and 2048 total visited
		// entries per folder to cap worst-case cost on huge repos.
		let WorkspacePatterns = {
			let AppState = &EnvironmentForActivation.ApplicationState;
			let Folders:Vec<std::path::PathBuf> = AppState
				.Workspace
				.WorkspaceFolders
				.lock()
				.ok()
				.map(|Guard| {
					Guard
						.iter()
						.filter_map(|Folder| Folder.URI.to_file_path().ok())
						.collect::<Vec<_>>()
				})
				.unwrap_or_default();

			let Patterns:Vec<String> = AppState
				.Extension
				.ScannedExtensions
				.ScannedExtensions
				.lock()
				.ok()
				.map(|Guard| {
					let mut Set:std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
					for Description in Guard.values() {
						if let Some(Events) = &Description.ActivationEvents {
							for Event in Events {
								if let Some(Pattern) = Event.strip_prefix("workspaceContains:") {
									Set.insert(Pattern.to_string());
								}
							}
						}
					}
					Set.into_iter().collect()
				})
				.unwrap_or_default();

			(Folders, Patterns)
		};

		let (WorkspaceFolders, Patterns):(Vec<std::path::PathBuf>, Vec<String>) = WorkspacePatterns;
		if !WorkspaceFolders.is_empty() && !Patterns.is_empty() {
			let Matched = FindMatchingWorkspaceContainsPatterns(&WorkspaceFolders, &Patterns);
			dev_log!(
				"exthost",
				"[CocoonManagement] workspaceContains scan: {} pattern(s) matched across {} folder(s)",
				Matched.len(),
				WorkspaceFolders.len()
			);
			for Pattern in Matched {
				let Event = format!("workspaceContains:{}", Pattern);
				if let Err(Error) = Vine::Client::SendRequest::Fn(
					&SideCarId,
					"$activateByEvent".to_string(),
					serde_json::json!({ "activationEvent": Event }),
					30_000,
				)
				.await
				{
					dev_log!(
						"cocoon",
						"warn: [CocoonManagement] $activateByEvent({}) failed: {}",
						Event,
						Error
					);
				}
			}
		}

		// Phase 3: onStartupFinished. Fire after the `*` burst has had a
		// moment to complete so late-binding extensions layered on top
		// of startup contributions resolve in the expected order.
		sleep(Duration::from_millis(2_000)).await;
		if let Err(Error) = Vine::Client::SendRequest::Fn(
			&SideCarId,
			"$activateByEvent".to_string(),
			serde_json::json!({ "activationEvent": "onStartupFinished" }),
			30_000,
		)
		.await
		{
			dev_log!(
				"cocoon",
				"warn: [CocoonManagement] $activateByEvent(onStartupFinished) failed: {}",
				Error
			);
		} else {
			dev_log!("cocoon", "[CocoonManagement] onStartupFinished activation triggered");
		}
	});

	// Store process handle for health monitoring and management
	{
		let mut state = COCOON_STATE.lock().await;
		state.ChildProcess = Some(ChildProcess);
		state.IsRunning = true;
		state.StartTime = Some(tokio::time::Instant::now());
		dev_log!("cocoon", "[CocoonManagement] Process state updated: Running");
	}

	// Reset health monitor on successful initialization
	{
		let mut health = COCOON_HEALTH.lock().await;
		health.clear_issues();
		dev_log!("cocoon", "[CocoonManagement] Health monitor reset to active state");
	}

	// Start background health monitoring
	let state_clone = Arc::clone(&COCOON_STATE);
	tokio::spawn(monitor_cocoon_health_task(state_clone));
	dev_log!("cocoon", "[CocoonManagement] Background health monitoring started");

	Ok(())
}

/// Background task that monitors Cocoon process health and logs crashes.
///
/// Once the child process has exited (or never existed), the monitor no
/// longer has anything useful to say - it exits quietly instead of
/// flooding the log with "No Cocoon process to monitor" every 5s, which
/// was rendering the dev log unreadable after any Cocoon crash.
async fn monitor_cocoon_health_task(state:Arc<Mutex<CocoonProcessState>>) {
	loop {
		tokio::time::sleep(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECONDS)).await;

		let mut state_guard = state.lock().await;

		// Check if we have a child process to monitor
		if state_guard.ChildProcess.is_some() {
			// Get process ID before checking status
			let process_id = state_guard.ChildProcess.as_ref().map(|c| c.id().unwrap_or(0));

			// Check if process is still running
			let exit_status = {
				let child = state_guard.ChildProcess.as_mut().unwrap();
				child.try_wait()
			};

			match exit_status {
				Ok(Some(exit_code)) => {
					// Process has exited (crashed or terminated)
					let uptime = state_guard.StartTime.map(|t| t.elapsed().as_secs()).unwrap_or(0);
					let exit_code_num = exit_code.code().unwrap_or(-1);
					dev_log!(
						"cocoon",
						"warn: [CocoonHealth] Cocoon process crashed [PID: {}] [Exit Code: {}] [Uptime: {}s]",
						process_id.unwrap_or(0),
						exit_code_num,
						uptime
					);

					// Update state
					state_guard.IsRunning = false;
					state_guard.ChildProcess = None;
					COCOON_PID.store(0, std::sync::atomic::Ordering::Relaxed);

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCrashed (Exit code: {})", exit_code_num)));
						dev_log!("cocoon", "warn: [CocoonHealth] Health score: {}", health.health_score);
					}

					// Log that automatic restart would be needed
					dev_log!(
						"cocoon",
						"warn: [CocoonHealth] CRASH DETECTED: Cocoon process has crashed and must be restarted \
						 manually or via application reinitialization"
					);
				},
				Ok(None) => {
					// Process is still running
					dev_log!(
						"cocoon",
						"[CocoonHealth] Cocoon process is healthy [PID: {}]",
						process_id.unwrap_or(0)
					);
				},
				Err(e) => {
					// Error checking process status
					dev_log!("cocoon", "warn: [CocoonHealth] Error checking process status: {}", e);

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCheckError: {}", e)));
					}
				},
			}
		} else {
			// No child process exists - log exactly once, then exit the
			// monitor loop. Prior behaviour: flood the log with
			// "No Cocoon process to monitor" every 5s forever after a
			// crash, making the dev log unreadable. A future respawn will
			// spawn a fresh monitor via `StartCocoon`.
			dev_log!("cocoon", "[CocoonHealth] No Cocoon process to monitor - exiting monitor loop");
			drop(state_guard);
			return;
		}
	}
}

/// Atom I6: post-shutdown hard-kill. Called by RuntimeShutdown after the
/// `$shutdown` gRPC notification has been sent (and either succeeded or
/// timed out). Grabs the stored `Child` handle and force-terminates it if
/// still alive, then resets COCOON_STATE. This plugs the "Mountain exits
/// cleanly but child stays running" leak that leads to zombie-Cocoon
/// zombies holding the gRPC port.
///
/// Call AFTER the graceful $shutdown attempt - we don't want to race the
/// child's own cleanup. Safe to call with no stored child (no-op).
pub async fn HardKillCocoon() {
	let mut State = COCOON_STATE.lock().await;
	if let Some(mut Child) = State.ChildProcess.take() {
		let Pid = Child.id().unwrap_or(0);
		match Child.try_wait() {
			Ok(Some(_Status)) => {
				dev_log!("cocoon", "[CocoonShutdown] Child PID {} already exited; clearing handle.", Pid);
			},
			Ok(None) => {
				dev_log!(
					"cocoon",
					"[CocoonShutdown] Child PID {} still alive after $shutdown; sending SIGKILL.",
					Pid
				);
				if let Err(Error) = Child.start_kill() {
					dev_log!("cocoon", "warn: [CocoonShutdown] start_kill failed on PID {}: {}", Pid, Error);
				}
				// Best-effort wait so the OS reaps and frees the port.
				let _ = tokio::time::timeout(std::time::Duration::from_secs(2), Child.wait()).await;
			},
			Err(Error) => {
				dev_log!("cocoon", "warn: [CocoonShutdown] try_wait failed on PID {}: {}", Pid, Error);
			},
		}
	}
	State.IsRunning = false;
}

/// Atom I6: pre-boot sweep. TCP-probe the Cocoon gRPC port and kill any
/// stale process still bound to it. Prevents the EADDRINUSE cascade that
/// leaves the extension host in degraded mode when a prior Mountain exited
/// without cleaning up its child.
///
/// Behaviour:
/// - If the port answers a TCP connect, assume an owner is listening.
/// - Use `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` (macOS/Linux) to resolve the
///   PID. `lsof` is ubiquitous on macOS/Linux and doesn't require root for
///   local user-owned processes.
/// - SIGTERM first, 500ms grace window, then SIGKILL if still alive.
/// - Logs every step via `dev_log!("cocoon", …)` so the sweep is visible in
///   Mountain.dev.log without parsing stderr.
/// - Best-effort: failures don't abort Mountain boot. A real EADDRINUSE later
///   will surface via Cocoon's own bootstrap error.
fn SweepStaleCocoon(Port:u16) {
	use std::{net::TcpStream, time::Duration};

	let Addr = format!("127.0.0.1:{}", Port);

	// Cheap liveness probe. Timeout is aggressive - zombie ports answer
	// immediately; a clean port is ECONNREFUSED and returns instantly.
	let Probe =
		TcpStream::connect_timeout(&Addr.parse().expect("valid socket addr literal"), Duration::from_millis(200));
	if Probe.is_err() {
		dev_log!("cocoon", "[CocoonSweep] Port {} is clean (no prior listener).", Port);
		return;
	}

	dev_log!(
		"cocoon",
		"[CocoonSweep] Port {} has a listener - attempting to resolve owner via lsof.",
		Port
	);

	// `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` → one PID per line.
	let LsofOutput = std::process::Command::new("lsof")
		.args(["-nP", &format!("-iTCP:{}", Port), "-sTCP:LISTEN", "-t"])
		.output();

	let Output = match LsofOutput {
		Ok(O) => O,
		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] lsof unavailable ({}). Skipping sweep; Cocoon spawn may fail with EADDRINUSE.",
				Error
			);
			return;
		},
	};

	if !Output.status.success() {
		dev_log!("cocoon", "warn: [CocoonSweep] lsof exited non-zero. Skipping sweep.");
		return;
	}

	let Stdout = String::from_utf8_lossy(&Output.stdout);
	let Pids:Vec<i32> = Stdout.lines().filter_map(|L| L.trim().parse::<i32>().ok()).collect();

	if Pids.is_empty() {
		dev_log!(
			"cocoon",
			"warn: [CocoonSweep] Port {} answered but lsof found no LISTEN PID - giving up.",
			Port
		);
		return;
	}

	// Guard against self-kill. Mountain currently binds 50051, not Cocoon's
	// 50052, but belt-and-braces for future refactors.
	let SelfPid = std::process::id() as i32;
	for Pid in Pids {
		if Pid == SelfPid {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] Port {} owned by Mountain itself (PID {}); refusing to kill.",
				Port,
				Pid
			);
			continue;
		}
		dev_log!("cocoon", "[CocoonSweep] Killing stale PID {} (SIGTERM).", Pid);
		let _ = std::process::Command::new("kill").arg(Pid.to_string()).status();
		std::thread::sleep(Duration::from_millis(500));
		// Recheck - if still alive, escalate.
		let StillAlive = std::process::Command::new("kill")
			.args(["-0", &Pid.to_string()])
			.status()
			.map(|S| S.success())
			.unwrap_or(false);
		if StillAlive {
			dev_log!("cocoon", "warn: [CocoonSweep] PID {} survived SIGTERM; sending SIGKILL.", Pid);
			let _ = std::process::Command::new("kill").args(["-9", &Pid.to_string()]).status();
			std::thread::sleep(Duration::from_millis(200));
		}
		dev_log!("cocoon", "[CocoonSweep] PID {} reaped.", Pid);
	}
}

/// Return the subset of `Patterns` for which at least one workspace folder
/// contains a matching file or directory. Patterns are interpreted the same
/// way VS Code does for `workspaceContains:<pattern>` activation events:
///
/// - A bare filename (no slash, no wildcards) matches an entry with that name
///   at the workspace root (e.g. `package.json`).
/// - A path with slashes but no wildcards matches a direct descendant relative
///   to the root (e.g. `.vscode/launch.json`).
/// - A glob with `**/` prefix matches any descendant up to a bounded depth.
/// - Any other wildcard form is matched via a simple segment-by-segment walk
///   honouring `*` (single segment) and `**` (any number of segments).
///
/// Matching is bounded to depth 3 and 4096 total directory entries per
/// workspace root to keep the cost sub-100 ms on large monorepos. Anything
/// deeper is rare for activation-event triggers; the trade-off is
/// documented in VS Code's own `ExtensionService.scanExtensions`.
fn FindMatchingWorkspaceContainsPatterns(Folders:&[std::path::PathBuf], Patterns:&[String]) -> Vec<String> {
	use std::collections::HashSet;

	const MAX_DEPTH:usize = 3;
	const MAX_ENTRIES_PER_ROOT:usize = 4096;

	let mut Matched:HashSet<String> = HashSet::new();
	for Folder in Folders {
		if !Folder.is_dir() {
			continue;
		}
		// Collect up to MAX_ENTRIES_PER_ROOT paths relative to the folder.
		let mut Entries:Vec<String> = Vec::new();
		let mut Stack:Vec<(std::path::PathBuf, usize)> = vec![(Folder.clone(), 0)];
		while let Some((Current, Depth)) = Stack.pop() {
			if Entries.len() >= MAX_ENTRIES_PER_ROOT {
				break;
			}
			let ReadDirResult = std::fs::read_dir(&Current);
			let ReadDir = match ReadDirResult {
				Ok(R) => R,
				Err(_) => continue,
			};
			for Entry in ReadDir.flatten() {
				if Entries.len() >= MAX_ENTRIES_PER_ROOT {
					break;
				}
				let Path = Entry.path();
				let Relative = match Path.strip_prefix(Folder) {
					Ok(R) => R.to_string_lossy().replace('\\', "/"),
					Err(_) => continue,
				};
				let IsDir = Entry.file_type().map(|T| T.is_dir()).unwrap_or(false);
				Entries.push(Relative.clone());
				if IsDir && Depth + 1 < MAX_DEPTH {
					Stack.push((Path, Depth + 1));
				}
			}
		}

		for Pattern in Patterns {
			if Matched.contains(Pattern) {
				continue;
			}
			if PatternMatchesAnyEntry(Pattern, &Entries) {
				Matched.insert(Pattern.clone());
			}
		}
	}
	Matched.into_iter().collect()
}

/// Very small glob-matcher scoped to VS Code `workspaceContains:` syntax.
/// Supports literal paths, `*` (one path segment), and `**` (zero or more
/// segments). Case-sensitive per the VS Code spec.
fn PatternMatchesAnyEntry(Pattern:&str, Entries:&[String]) -> bool {
	let HasWildcard = Pattern.contains('*') || Pattern.contains('?');
	if !HasWildcard {
		return Entries.iter().any(|E| E == Pattern);
	}
	let PatternSegments:Vec<&str> = Pattern.split('/').collect();
	Entries
		.iter()
		.any(|E| SegmentMatch(&PatternSegments, &E.split('/').collect::<Vec<_>>()))
}

fn SegmentMatch(Pattern:&[&str], Entry:&[&str]) -> bool {
	if Pattern.is_empty() {
		return Entry.is_empty();
	}
	let Head = Pattern[0];
	if Head == "**" {
		// `**` matches zero or more segments. Try consuming 0..=entry.len().
		for Consumed in 0..=Entry.len() {
			if SegmentMatch(&Pattern[1..], &Entry[Consumed..]) {
				return true;
			}
		}
		return false;
	}
	if Entry.is_empty() {
		return false;
	}
	if SingleSegmentMatch(Head, Entry[0]) {
		return SegmentMatch(&Pattern[1..], &Entry[1..]);
	}
	false
}

fn SingleSegmentMatch(Pattern:&str, Segment:&str) -> bool {
	if Pattern == "*" {
		return true;
	}
	if !Pattern.contains('*') && !Pattern.contains('?') {
		return Pattern == Segment;
	}
	// Minimal star-glob on a single segment: split by '*' and check each
	// fragment appears in order. Doesn't support `?` (rare in
	// workspaceContains patterns); unsupported glob chars fall through to
	// literal equality.
	let Fragments:Vec<&str> = Pattern.split('*').collect();
	let mut Cursor = 0usize;
	for (Index, Fragment) in Fragments.iter().enumerate() {
		if Fragment.is_empty() {
			continue;
		}
		if Index == 0 {
			if !Segment[Cursor..].starts_with(Fragment) {
				return false;
			}
			Cursor += Fragment.len();
			continue;
		}
		match Segment[Cursor..].find(Fragment) {
			Some(Offset) => Cursor += Offset + Fragment.len(),
			None => return false,
		}
	}
	if let Some(Last) = Fragments.last()
		&& !Last.is_empty()
	{
		return Segment.ends_with(Last);
	}
	true
}
