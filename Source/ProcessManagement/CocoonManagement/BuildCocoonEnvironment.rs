//! Build the complete environment variable map for the Cocoon subprocess:
//! VS Code pipe-logging vars, gRPC ports, PATH/HOME passthrough, the
//! `Product*`/`Tier*`/`Network*` prefixes, the PascalCase Land allow-list,
//! and NODE_ENV / TAURI_ENV_DEBUG.

use std::collections::HashMap;

pub(crate) fn Fn() -> HashMap<String, String> {
	const LAND_ENV_ALLOW_LIST:&[&str] = &[
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
		"Capture",
		"Pipe",
		"Emit",
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
		"DebugServer",
		"DebugServerPortMountain",
		"DebugServerPortCocoon",
	];

	let mut Env = HashMap::new();

	Env.insert("VSCODE_PIPE_LOGGING".into(), "true".into());

	Env.insert("VSCODE_VERBOSE_LOGGING".into(), "true".into());

	Env.insert("VSCODE_PARENT_PID".into(), std::process::id().to_string());

	Env.insert("MOUNTAIN_GRPC_PORT".into(), super::MOUNTAIN_GRPC_PORT.to_string());

	Env.insert("COCOON_GRPC_PORT".into(), super::COCOON_GRPC_PORT.to_string());

	// B7-S6: WebSocket transport config. Only pass port+secret to Cocoon
	// when TierWebSocket is not Disabled - otherwise Cocoon starts the WS
	// server unconditionally and Sky tries to connect, causing CSP errors.
	// Runtime env wins for dev overrides; the compile-time value baked
	// from `.env.Land` by build.rs is the fallback (same pattern as
	// TierCommandEventBroadcast in mod.rs) - reading only the process env
	// silently disabled WS for every launch that did not re-export
	// TierWebSocket in the shell.
	let TierWS = std::env::var("TierWebSocket").unwrap_or_else(|_| env!("TierWebSocket", "Disabled").to_string());

	if TierWS != "Disabled" {
		super::InitializeWsConfig();

		let WsPort = super::WsPort();

		if WsPort > 0 {
			Env.insert("COCOON_WS_PORT".into(), WsPort.to_string());

			Env.insert("COCOON_WS_SECRET".into(), super::WsSecretHex());
		}
	}

	for Key in ["PATH", "HOME"] {
		if let Ok(V) = std::env::var(Key) {
			Env.insert(Key.into(), V);
		}
	}

	for (Key, Value) in std::env::vars() {
		if Key.starts_with("Product")
			|| Key.starts_with("Tier")
			|| Key.starts_with("Network")
			|| LAND_ENV_ALLOW_LIST.contains(&Key.as_str())
		{
			Env.insert(Key, Value);
		}
	}

	for Key in ["NODE_ENV", "TAURI_ENV_DEBUG"] {
		if let Ok(V) = std::env::var(Key) {
			Env.insert(Key.into(), V);
		}
	}

	Env
}
