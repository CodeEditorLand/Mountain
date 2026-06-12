//! # Air Start Module
//!
//! Spawns and connects to the Air daemon sidecar.
//!
//! Mirror of `CocoonStart.rs`. Air is the long-lived background daemon
//! responsible for updates, downloads, crypto signing, file indexing,
//! system monitoring, and other off-the-hot-path work that should not
//! tax the workbench. Mountain spawns Air at boot, connects via gRPC
//! over `[::1]:50053`, and registers it as a sidecar in the same
//! Vine pool as Cocoon.
//!
//! Lifecycle parity with Cocoon:
//!
//!   - Resolve binary path from the Tauri sidecar resolver. The Air binary
//!     ships next to Mountain in the bundle (release builds); in dev mode the
//!     Cargo target dir is searched.
//!   - Spawn as a background tokio task with stdout/stderr captured.
//!   - Wait for the gRPC server to become available, then create an `AirClient`
//!     and store it in the environment for handlers to consume.
//!   - On failure: log a degraded-mode warning and return Ok(()) - the
//!     workbench works without Air, just without update / index /
//!     system-monitor capability.

use std::sync::Arc;

use tauri::AppHandle;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

/// Default gRPC address used by Air. Mirror of
/// `AirClient::DEFAULT_AIR_SERVER_ADDRESS` for the connect step.
const AIR_GRPC_ADDRESS:&str = "[::1]:50053";

/// Spawn and connect to the Air daemon. Returns Ok(()) regardless of
/// outcome - Air is non-essential for workbench operation; Mountain
/// gracefully degrades when Air is unavailable.
///
/// Spawn is gated on:
///   - The `AirIntegration` Cargo feature (compile-time).
///   - The `Spawn` env var (runtime; mirrors `CocoonStart` semantics).
pub async fn Fn(_ApplicationHandle:&AppHandle, _Environment:&Arc<MountainEnvironment>) -> Result<(), String> {
	// Atom N1 mirror: respect the `Spawn=false` env that disables
	// sidecar spawn for tests and the smallest-shippable-surface
	// Mountain-only profile.
	if matches!(std::env::var("Spawn").as_deref(), Ok("0") | Ok("false")) {
		dev_log!("grpc", "[AirStart] Skipping Air spawn (Spawn=false)");

		return Ok(());
	}

	#[cfg(feature = "AirIntegration")]
	{
		LaunchAndConnectAir(_ApplicationHandle.clone(), _Environment.clone()).await
	}

	#[cfg(not(feature = "AirIntegration"))]
	{
		dev_log!(
			"grpc",
			"[AirStart] AirIntegration feature disabled; skipping spawn (workbench runs without Air)"
		);

		Ok(())
	}
}

#[cfg(feature = "AirIntegration")]
async fn LaunchAndConnectAir(ApplicationHandle:AppHandle, _Environment:Arc<MountainEnvironment>) -> Result<(), String> {
	use std::path::PathBuf;

	use tauri::Manager;

	dev_log!("grpc", "[AirStart] Resolving Air sidecar binary path...");

	// Air builds into its own per-element Target dir
	// (`Element/Air/.cargo/config.toml`), profile-matched to Mountain's.
	let Profile = if cfg!(debug_assertions) { "debug" } else { "release" };

	// Try the Tauri sidecar resolver first (release / bundled - SignBundle.sh
	// copies Air into Contents/Resources). Dev fallbacks: explicit
	// CARGO_TARGET_DIR, repo layout relative to the running Mountain binary
	// (CWD-independent), then CWD-relative for `cargo run` from `Land/`.
	let BinaryPath:Option<PathBuf> = ApplicationHandle
		.path()
		.resolve("Air", tauri::path::BaseDirectory::Resource)
		.ok()
		.filter(|P| P.exists())
		.or_else(|| {
			let Candidate = std::env::var("CARGO_TARGET_DIR").map(PathBuf::from).ok()?.join("Air");

			Candidate.exists().then_some(Candidate)
		})
		.or_else(|| {
			// Mountain exe lives at Element/Mountain/Target/<profile>/<name>;
			// hop to the sibling element: Element/Air/Target/<profile>/Air.
			let ExeDir = std::env::current_exe().ok()?.parent()?.to_path_buf();

			let Candidate = ExeDir.join(format!("../../../Air/Target/{Profile}/Air"));

			Candidate.exists().then_some(Candidate)
		})
		.or_else(|| {
			let Candidate = PathBuf::from(format!("Element/Air/Target/{Profile}/Air"));

			Candidate.exists().then_some(Candidate)
		});

	let BinaryPath = match BinaryPath {
		Some(P) => P,

		None => {
			dev_log!(
				"grpc",
				"warn: [AirStart] Air binary not found in resources or target/debug; running without Air"
			);

			return Ok(());
		},
	};

	dev_log!("grpc", "[AirStart] Spawning Air binary at: {}", BinaryPath.display());

	// Spawn detached so Air's lifecycle is independent of Mountain's
	// boot path. Mountain holds no Child handle - Air manages its own
	// shutdown via SIGTERM from the OS or its own gRPC `Shutdown` RPC.
	let SpawnResult = tokio::process::Command::new(&BinaryPath)
		.env("AIR_GRPC_ADDRESS", AIR_GRPC_ADDRESS)

		// Air's Configuration layer reads `AIR_GRPC_BIND_ADDRESS` (prefix
		// `AIR_` + `grpc.bind_address`); `AIR_GRPC_ADDRESS` above is kept
		// for the AirClient-side convention.
		.env("AIR_GRPC_BIND_ADDRESS", AIR_GRPC_ADDRESS)
		.env(
			"AIR_LOG_DIR",

			std::env::var("AIR_LOG_DIR").unwrap_or_else(|_| "/tmp/air-log".to_string()),
		)
		.stdin(std::process::Stdio::null())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn();

	let mut Child = match SpawnResult {
		Ok(C) => C,

		Err(Error) => {
			dev_log!("grpc", "warn: [AirStart] Failed to spawn Air ({}); running without Air", Error);

			return Ok(());
		},
	};

	let AirPid = Child.id();

	dev_log!("grpc", "[AirStart] Air spawned successfully (pid={:?})", AirPid);

	// Drain Air's stdout/stderr into Mountain's dev log so the user
	// can diagnose Air-side issues from a single log stream.
	if let Some(Stdout) = Child.stdout.take() {
		tokio::spawn(async move {
			use tokio::io::{AsyncBufReadExt, BufReader};

			let mut Reader = BufReader::new(Stdout).lines();

			while let Ok(Some(Line)) = Reader.next_line().await {
				dev_log!("grpc", "[Air stdout] {}", Line);
			}
		});
	}

	if let Some(Stderr) = Child.stderr.take() {
		tokio::spawn(async move {
			use tokio::io::{AsyncBufReadExt, BufReader};

			let mut Reader = BufReader::new(Stderr).lines();

			while let Ok(Some(Line)) = Reader.next_line().await {
				dev_log!("grpc", "[Air stderr] {}", Line);
			}
		});
	}

	// Reap the child in a detached task so the OS doesn't keep a
	// zombie around when Air exits.
	tokio::spawn(async move {
		match Child.wait().await {
			Ok(Status) => dev_log!("grpc", "[AirStart] Air exited (status={:?})", Status),
			Err(Error) => dev_log!("grpc", "warn: [AirStart] Air wait error: {}", Error),
		}
	});

	// Connect via Vine. Air's gRPC server takes ~150 ms to become
	// listenable; ConnectToSideCar handles the retry loop.
	let SideCarIdentifier = "air-main".to_string();

	let Address = format!("http://{}", AIR_GRPC_ADDRESS);

	match ::Vine::Client::ConnectToSideCar::Fn(SideCarIdentifier.clone(), Address.clone()).await {
		Ok(()) => {
			dev_log!("grpc", "[AirStart] Air gRPC connection established at {}", Address);
		},

		Err(Error) => {
			dev_log!(
				"grpc",
				"warn: [AirStart] Air spawned but gRPC connect failed ({}); workbench continues in degraded mode",
				Error
			);
		},
	}

	Ok(())
}
