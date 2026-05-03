#![allow(non_snake_case)]
use std::fs::read_to_string;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
struct Toml {
	package:Package,
}

#[derive(Deserialize)]
struct Package {
	version:String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	if !tauri_build::is_dev() {
		println!("cargo:rerun-if-changed=Cargo.toml");

		println!("cargo:rerun-if-changed=tauri.conf.json");

		println!("cargo:rerun-if-changed=tauri.conf.json5");

		let Version = toml::from_str::<Toml>(&read_to_string("Cargo.toml")?)?.package.version;

		let File = if std::path::Path::new("tauri.conf.json5").exists() {
			"tauri.conf.json5"
		} else {
			"tauri.conf.json"
		};

		let Content = read_to_string(File)?;

		let mut Tauri:Value = match json5::from_str(&Content) {
			Ok(Value) => Value,
			Err(_) => serde_json::from_str(&Content)?,
		};

		if let Some(Entry) = Tauri.get_mut("version") {
			*Entry = Value::String(Version.clone());
		}

		let mut Serializer =
			serde_json::Serializer::with_formatter(Vec::new(), serde_json::ser::PrettyFormatter::with_indent(b"\t"));

		Tauri.serialize(&mut Serializer)?;

		std::fs::write(File, String::from_utf8(Serializer.into_inner())?)?;

		println!("cargo:rustc-env=CARGO_PKG_VERSION={}", Version);
	}

	println!("cargo:rerun-if-changed=Proto/Vine.proto");

	tonic_prost_build::configure()
		.build_server(true)
		.build_client(true)
		.out_dir("Source/Vine/Generated")
		.compile_well_known_types(true)
		.compile_protos(&["Proto/Vine.proto"], &["Proto"])?;

	PropagateTierGating();

	PropagateProfileSentinel();

	PropagatePostHogSentinel();

	tauri_build::build();

	Ok(())
}

// ===========================================================================
// Profile sentinel: Build.sh exports `Browser=true` / `Mountain=true` /
// `Electron=true` / `Bundle` / `Compiler` / `Profile` into the shell
// that invokes cargo. These shell env vars don't survive to the resulting
// binary; only `cargo:rustc-env` does. Bake a PascalCase Land env set
// (`Profile`, `Pack`, `Bundle`, `Compiler`) into the binary so
// `option_env!("Profile")` resolves after launch without depending on
// the shell the user runs the binary from. Legacy `LAND_*` names were
// retired in the 2026-04-29 PascalCase migration.
//
// Follow-up to playbook item #3 - previously the sentinel logged
// `Active profile=unknown` because it ran `std::env::var` at runtime.
// ===========================================================================

fn PropagateProfileSentinel() {
	let Browser = std::env::var("Browser").unwrap_or_default();

	let MountainProfile = std::env::var("Mountain").unwrap_or_default();

	let Electron = std::env::var("Electron").unwrap_or_default();

	let Bundle = std::env::var("Bundle").unwrap_or_default();

	let Compiler = std::env::var("Compiler").unwrap_or_default();

	// rerun on any of these changing so incremental builds pick up a
	// `Compiler=rest` flip without a clean.
	for Key in ["Browser", "Mountain", "Electron", "Bundle", "Compiler", "Profile"] {
		println!("cargo:rerun-if-env-changed={Key}");
	}

	let Named = std::env::var("Profile").unwrap_or_else(|_| {
		if Electron == "true" {
			if Compiler.eq_ignore_ascii_case("rest") {
				"debug-electron-rest".into()
			} else {
				"debug-electron".into()
			}
		} else if MountainProfile == "true" {
			"debug-mountain".into()
		} else if Browser == "true" {
			"debug".into()
		} else {
			"unknown".into()
		}
	});

	let Workbench = if Electron == "true" {
		"Electron"
	} else if MountainProfile == "true" {
		"Mountain"
	} else if Browser == "true" {
		"Browser"
	} else {
		"Unknown"
	};

	let CompilerLabel = if Compiler.is_empty() { "default" } else { Compiler.as_str() };

	println!("cargo:rustc-env=Profile={Named}");

	println!("cargo:rustc-env=Pack={Workbench}");

	println!("cargo:rustc-env=Bundle={Bundle}");

	println!("cargo:rustc-env=Compiler={CompilerLabel}");
}

// ===========================================================================
// PostHog + OTLP sentinel: `.env.Land.PostHog` exposes Authorize /
// Beam / Report / Brand and the OTLP / Disable / Trace overlay knobs.
// TierEnvironment.sh sources that overlay before cargo runs, so values are
// available here. Bake them as `cargo:rustc-env` so Mountain reads a single
// source of truth across every build profile without a hardcoded const.
// ===========================================================================

fn PropagatePostHogSentinel() {
	for Key in [
		"Authorize",
		"Beam",
		"Report",
		"Brand",
		"OTLPEndpoint",
		"OTLPEnabled",
		"Capture",
		"Trace",
	] {
		println!("cargo:rerun-if-env-changed={Key}");
	}

	let Key = std::env::var("Authorize").unwrap_or_else(|_| "".into());

	let Host = std::env::var("Beam").unwrap_or_else(|_| "https://eu.i.posthog.com".into());

	let Enabled = std::env::var("Report").unwrap_or_else(|_| "true".into());

	let DistinctId = std::env::var("Brand").unwrap_or_default();

	let OTLPEndpoint = std::env::var("OTLPEndpoint").unwrap_or_else(|_| "http://127.0.0.1:4318".into());

	let OTLPEnabled = std::env::var("OTLPEnabled").unwrap_or_else(|_| "true".into());

	let TelemetryCapture = std::env::var("Capture").unwrap_or_else(|_| "true".into());

	let TraceFilter = std::env::var("Trace").unwrap_or_else(|_| "all".into());

	println!("cargo:rustc-env=Authorize={Key}");

	println!("cargo:rustc-env=Beam={Host}");

	println!("cargo:rustc-env=Report={Enabled}");

	println!("cargo:rustc-env=Brand={DistinctId}");

	println!("cargo:rustc-env=OTLPEndpoint={OTLPEndpoint}");

	println!("cargo:rustc-env=OTLPEnabled={OTLPEnabled}");

	println!("cargo:rustc-env=Capture={TelemetryCapture}");

	println!("cargo:rustc-env=Trace={TraceFilter}");
}

// ===========================================================================
// Tier-gating: read `.env.Land` at the workspace root, expose every value as
// `cargo:rustc-env=Tier<Capability>=<Value>` so `env!("TierFileSystem")` works
// at runtime, and activate matching Cargo features so `#[cfg(feature = "…")]`
// arms compile in.
//
// The feature-name whitelist below is intentional: an unknown pair in
// `.env.Land` produces a `cargo:warning` but never activates anything,
// so typos fail loud at `cargo build` instead of silently at runtime.
//
// See `Documentation/GitHub/Workflow/TierGatedImplementationSelection.md`
// for the full build-time propagation workflow.
// ===========================================================================

fn PropagateTierGating() {
	// Emit defaults for every tier variable FIRST so that `env!("Tier…")` at
	// compile time always resolves, even when `.env.Land` is absent or when a
	// file exists but omits a key. File values emitted later override - Cargo
	// honours the last `rustc-env` for a given key.
	EmitTierDefaults();

	let Manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
	let ManifestPath = std::path::PathBuf::from(Manifest);

	// Mountain/Cargo.toml is at Land/Element/Mountain/Cargo.toml - three
	// ancestors up (Mountain → Element → Land) is the Land monorepo root,
	// four is the repo root. Try both to find the env file.
	let Ancestors:Vec<_> = ManifestPath.ancestors().take(5).map(|p| p.to_path_buf()).collect();

	for Base in &Ancestors {
		for Candidate in [".env.Land", ".env.Land.Sample"] {
			let Full = Base.join(Candidate);
			if !Full.exists() {
				continue;
			}
			println!("cargo:rerun-if-changed={}", Full.display());
			let Contents = match std::fs::read_to_string(&Full) {
				Ok(text) => text,
				Err(error) => {
					println!("cargo:warning=Failed to read {}: {}", Full.display(), error);
					return;
				},
			};
			ApplyEnvFile(&Full, &Contents);
			// Stop after the first env file we find - subsequent ones would
			// duplicate directives (defaults are already emitted upstream).
			return;
		}
	}
}

/// Parse a loaded `.env.Land` file body, emitting `rustc-env` overrides and
/// `rustc-cfg` feature flags for recognised tier values.
fn ApplyEnvFile(Path:&std::path::Path, Contents:&str) {
	for Line in Contents.lines() {
		let Trimmed = Line.trim();
		if Trimmed.is_empty() || Trimmed.starts_with('#') {
			continue;
		}
		let Pair = Trimmed.split_once('=');
		let (Key, Value) = match Pair {
			Some(pair) => pair,
			None => continue,
		};
		let Key = Key.trim();
		let Value = Value.trim().trim_matches('"');

		if !Key.starts_with("Tier") {
			continue;
		}

		// Override the default emitted by EmitTierDefaults. Multiple
		// rustc-env directives for the same key: last wins.
		println!("cargo:rustc-env={}={}", Key, Value);

		let FeatureName = format!("{}{}", Key, Value);
		if IsDeclaredTierFeature(&FeatureName) {
			println!("cargo:rustc-cfg=feature=\"{}\"", FeatureName);
		} else if IsDefaultTierValue(Key, Value) {
			// Default-tier values (GRPC, Layer2, Standard, …) do not need
			// Cargo features - they are the compiled-in baseline. Silent.
		} else {
			println!(
				"cargo:warning={}={} declared in {} but has no matching Cargo feature - update IsDeclaredTierFeature \
				 or Cargo.toml",
				Key,
				Value,
				Path.display()
			);
		}
	}
}

/// Emit compile-time defaults for every tier variable. Called unconditionally
/// so `env!(...)` in LandFixTier resolves even in a clean checkout with no
/// `.env.Land` file.
fn EmitTierDefaults() {
	for (Key, Default) in [
		("TierRemoteProcedureCall", "GRPC"),
		("TierHTTPProxy", "HandRolled"),
		("TierLogger", "Standard"),
		("TierFileSystem", "Layer2"),
		("TierFindFiles", "Layer3"),
		("TierGlob", "JavaScript"),
		("TierFileWatcher", "Stub"),
		("TierSchemeAssets", "Embedded"),
		("TierConfiguration", "Cache"),
		("TierDiagnostics", "Full"),
		("TierClipboard", "Layer3"),
		("TierOpenExternal", "Layer3"),
		("TierDocumentMirror", "Full"),
		("TierExtensionActivation", "Parallel8"),
		("TierExtensionScan", "Sequential"),
		("TierModuleCache", "Simple"),
		("TierTelemetry", "Synchronous"),
	] {
		println!("cargo:rustc-env={}={}", Key, Default);
	}
}

/// Whitelist of Rust-side tier feature names. An unknown combination in
/// `.env.Land` surfaces as a build warning rather than a silent no-op.
fn IsDeclaredTierFeature(Name:&str) -> bool {
	matches!(
		Name,
		"TierRemoteProcedureCallSharedMemory"
			| "TierHTTPProxyHyper"
			| "TierLoggerRing"
			| "TierFileSystemLayer4"
			| "TierFindFilesLayer4"
			| "TierGlobNative"
			| "TierFileWatcherLayer4"
			| "TierSchemeAssetsHybrid"
			| "TierConfigurationEager"
			| "TierDiagnosticsDelta"
			| "TierClipboardLayer4"
			| "TierOpenExternalLayer4"
			| "TierExtensionScanParallel"
	)
}

/// Whitelist of tier (Key, Value) pairs that are compiled-in defaults - they
/// don't activate a Cargo feature because they ARE the baseline. Keeping a
/// dedicated table (rather than one big negation) means typos in `.env.Land`
/// still surface as warnings; only the exact default-value spelling is
/// silent.
fn IsDefaultTierValue(Key:&str, Value:&str) -> bool {
	matches!(
		(Key, Value),
		("TierRemoteProcedureCall", "GRPC")
			| ("TierHTTPProxy", "HandRolled")
			| ("TierLogger", "Standard")
			| ("TierFileSystem", "Layer2" | "Layer3")
			| ("TierFindFiles", "Layer3")
			| ("TierGlob", "JavaScript")
			| ("TierFileWatcher", "Stub")
			| ("TierSchemeAssets", "Embedded" | "FileSystem")
			| ("TierConfiguration", "Cache")
			| ("TierDiagnostics", "Full")
			| ("TierClipboard", "Layer3" | "Layer5")
			| ("TierOpenExternal", "Layer3")
			| ("TierDocumentMirror", "Full" | "Lazy")
			| (
				"TierExtensionActivation",
				"Sequential" | "Parallel4" | "Parallel8" | "Parallel16"
			) | ("TierExtensionScan", "Sequential")
			| ("TierModuleCache", "Off" | "Simple" | "Shared")
			| ("TierTelemetry", "Synchronous" | "Batched" | "Off")
	)
}
