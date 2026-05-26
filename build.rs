#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments
)]

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

		// ---------------------------------------------------------------
		// TierSchemeAssets gate: mutate tauri.conf.json so that the right
		// frontendDist + bundle.resources are active at build time.
		//
		//  Embedded   → frontendDist: "../Sky/Target"
		//               resources: original small list (only Cocoon-needed
		//               files; everything else baked into the binary via
		//               EmbeddedAssets / asset_resolver).
		//
		//  FileSystem → frontendDist: null
		//               resources: full disk layout (vs/, _astro/, etc.)
		//               so Contents/Resources/ holds the complete tree and
		//               the localhost plugin / vscode-file scheme serve
		//               from disk via the static_root fallback.
		// ---------------------------------------------------------------
		// Priority: process env → .env.Land file → default "Embedded".
		// Reading the file directly guards against a stale shell env (e.g.
		// the previous tier was exported but .env.Land was since updated).
		// This matches the logic in PropagateTierGating which also walks up
		// from CARGO_MANIFEST_DIR to find the env file.
		let TierSchemeAssets = std::env::var("TierSchemeAssets")
			.unwrap_or_else(|_| ReadTierValueFromEnvFile("TierSchemeAssets").unwrap_or_else(|| "Embedded".into()));

		// Patch build.frontendDist
		if let Some(Build) = Tauri.get_mut("build") {
			let FrontendDist = if TierSchemeAssets == "FileSystem" {
				Value::Null
			} else {
				Value::String("../Sky/Target".into())
			};
			if let Some(B) = Build.as_object_mut() {
				B.insert("frontendDist".into(), FrontendDist);
			}
		}

		// Patch bundle.resources
		if let Some(Bundle) = Tauri.get_mut("bundle") {
			let Resources = if TierSchemeAssets == "FileSystem" {
				// Full disk layout - every file served from Contents/Resources/.
				serde_json::json!({
					"../Sky/Target/Browser": "Browser",
					"../Sky/Target/BrowserProxy": "BrowserProxy",
					"../Sky/Target/Bundled": "Bundled",
					"../Sky/Target/Electron": "Electron",
					"../Sky/Target/Favicon": "Favicon",
					"../Sky/Target/Isolation": "Isolation",
					"../Sky/Target/Manifest.json": "Manifest.json",
					"../Sky/Target/Mountain": "Mountain",
					"../Sky/Target/Static/Application/bootstrap-esm.js": "Static/Application/bootstrap-esm.js",
					"../Sky/Target/Static/Application/bootstrap-import.js": "Static/Application/bootstrap-import.js",
					"../Sky/Target/Static/Application/bootstrap-meta.js": "Static/Application/bootstrap-meta.js",
					"../Sky/Target/Static/Application/extensions": "Static/Application/extensions",
					"../Sky/Target/Static/Application/nls.keys.json": "Static/Application/nls.keys.json",
					"../Sky/Target/Static/Application/nls.messages.js": "Static/Application/nls.messages.js",
					"../Sky/Target/Static/Application/nls.messages.json": "Static/Application/nls.messages.json",
					"../Sky/Target/Static/Application/nls.metadata.json": "Static/Application/nls.metadata.json",
					"../Sky/Target/Static/Application/node_modules": "Static/Application/node_modules",
					"../Sky/Target/Static/Application/vs": "Static/Application/vs",
					"../Sky/Target/Worker.js": "Worker.js",
					"../Sky/Target/_astro": "_astro",
					"../Sky/Target/index.html": "index.html",
					"../Sky/Target/product.json": "product.json",
					"../Sky/Target/robots.txt": "robots.txt",
					"../Cocoon/Target/Bootstrap/Implementation/Cocoon": "Cocoon/Target/Bootstrap/Implementation/Cocoon",
					"../Sky/Target/extensions.manifest.json": "extensions.manifest.json",
					"Proto/Vine.proto": "Mountain/Proto/Vine.proto",
					"scripts/cocoon/bootstrap-fork.js": "scripts/cocoon/bootstrap-fork.js"
				})
			} else {
				// Embedded layout - only Cocoon-needed files on disk; all
				// workbench JS is Brotli-compressed into the binary.
				serde_json::json!({
					"../Sky/Target/Static/Application/bootstrap-esm.js": "bootstrap-esm.js",
					"../Sky/Target/Static/Application/bootstrap-import.js": "bootstrap-import.js",
					"../Sky/Target/Static/Application/bootstrap-meta.js": "bootstrap-meta.js",
					"../Sky/Target/Static/Application/extensions": "extensions",
					"../Sky/Target/Static/Application/nls.keys.json": "nls.keys.json",
					"../Sky/Target/Static/Application/nls.messages.js": "nls.messages.js",
					"../Sky/Target/Static/Application/nls.messages.json": "nls.messages.json",
					"../Sky/Target/Static/Application/nls.metadata.json": "nls.metadata.json",
					"../Sky/Target/Static/Application/node_modules": "node_modules",
					"../Sky/Target/product.json": "product.json",
					"../Cocoon/Target/Bootstrap/Implementation/Cocoon": "Cocoon/Target/Bootstrap/Implementation/Cocoon",
					"../Sky/Target/extensions.manifest.json": "extensions.manifest.json",
					"Proto/Vine.proto": "Mountain/Proto/Vine.proto",
					"scripts/cocoon/bootstrap-fork.js": "scripts/cocoon/bootstrap-fork.js"
				})
			};
			if let Some(B) = Bundle.as_object_mut() {
				B.insert("resources".into(), Resources);
			}
		}

		let mut Serializer =
			serde_json::Serializer::with_formatter(Vec::new(), serde_json::ser::PrettyFormatter::with_indent(b"	"));

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

	// Ensure extensions.manifest.json exists before tauri_build::build()
	// validates resource paths. Sky's BakeExtensionManifest integration writes
	// the real file during astro:build:done (part of beforeBuildCommand), so it
	// will be present for normal tauri builds. This stub covers the edge case of
	// a direct `cargo build -p Mountain` run without a prior Sky build. At
	// runtime LoadFromCache.rs falls back to a live scan when the array is empty.
	let ExtensionsManifest = std::path::Path::new("../Sky/Target/extensions.manifest.json");
	if !ExtensionsManifest.exists() {
		if let Some(Parent) = ExtensionsManifest.parent() {
			let _ = std::fs::create_dir_all(Parent);
		}
		// Write a valid empty CacheBlob - the parser expects
		// { version, count, extensions } not a bare array.
		let _ = std::fs::write(ExtensionsManifest, r#"{"version":1,"count":0,"extensions":[]}"#);
	}

	// Skip resource-path validation when generating docs. tauri_build::build()
	// checks that every [[bundle.resources]] path exists, but Sky assets
	// (bootstrap-meta.js etc.) are only present after a full Astro build.
	// Package.sh sets CARGO_BUILDING_DOCS=1 to signal this context.
	if std::env::var("CARGO_BUILDING_DOCS").is_err() {
		tauri_build::build();
	}

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
		"Pipe",
		"Emit",
		"Capture",
		"Trace",
	] {
		println!("cargo:rerun-if-env-changed={Key}");
	}

	let Key = std::env::var("Authorize").unwrap_or_else(|_| "".into());

	let Host = std::env::var("Beam").unwrap_or_else(|_| "https://eu.i.posthog.com".into());

	let Enabled = std::env::var("Report").unwrap_or_else(|_| "true".into());

	let DistinctId = std::env::var("Brand").unwrap_or_default();

	let Pipe = std::env::var("Pipe").unwrap_or_else(|_| "http://127.0.0.1:4318".into());

	let Emit = std::env::var("Emit").unwrap_or_else(|_| "true".into());

	let TelemetryCapture = std::env::var("Capture").unwrap_or_else(|_| "true".into());

	let TraceFilter = std::env::var("Trace").unwrap_or_else(|_| "all".into());

	println!("cargo:rustc-env=Authorize={Key}");

	println!("cargo:rustc-env=Beam={Host}");

	println!("cargo:rustc-env=Report={Enabled}");

	println!("cargo:rustc-env=Brand={DistinctId}");

	println!("cargo:rustc-env=Pipe={Pipe}");

	println!("cargo:rustc-env=Emit={Emit}");

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

			// Default-tier values (gRPC, Layer2, Standard, …) do not need
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
		("TierRemoteProcedureCall", "gRPC"),
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
		// Per-subsystem routing tiers (see .env.Land). All default to Mountain
		// except where the Atomic-branch baseline diverges (Tasks, Auth → Node;
		// WebSocket → Disabled until Mist server lands).
		("TierIPC", "Mountain"),
		("TierTerminal", "Mountain"),
		("TierSCM", "Mountain"),
		("TierDebug", "Mountain"),
		("TierLanguageFeatures", "Mountain"),
		("TierSearch", "Mountain"),
		("TierOutputChannel", "Mountain"),
		("TierNativeHost", "Mountain"),
		("TierTreeView", "Mountain"),
		("TierStorage", "Mountain"),
		("TierModel", "Mountain"),
		("TierTasks", "Node"),
		("TierAuth", "Node"),
		("TierEncryption", "Mountain"),
		("TierExtensionHost", "Process"),
		("TierWebSocket", "Disabled"),
		// TierCommandEventBroadcast - opt-in dual-emit on `Command.Execute`
		// gRPC arm so subscribers of `vscode.commands.onDidExecuteCommand`
		// see commands invoked from extension hosts (not just workbench).
		// Off by default because every gRPC executeCommand adds an extra
		// Vine notification roundtrip; flip to `On` when an extension
		// peer needs the event.
		("TierCommandEventBroadcast", "Off"),
	] {
		println!("cargo:rustc-env={}={}", Key, Default);
	}

	// Optional cfg flag for builds that omit the Cocoon Node.js host entirely.
	// Activated by setting `TierExtensionHost=WebWorker` in `.env.Land`; gates
	// out `CocoonManagement.rs` spawn logic via `#[cfg(not(no_node_host))]`.
	let TierExtensionHost = std::env::var("TierExtensionHost")
		.unwrap_or_else(|_| ReadTierValueFromEnvFile("TierExtensionHost").unwrap_or_else(|| "Process".into()));

	if TierExtensionHost == "WebWorker" {
		println!("cargo:rustc-cfg=no_node_host");
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
			// TierSchemeAssets non-default values:
			//   FileSystem → assets served from disk via static_root fallback;
			//                LocalhostPlugin.rs gates static_root on this feature.
			//   Hybrid     → reserved for future mixed embedding strategies.
			| "TierSchemeAssetsFileSystem"
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
		("TierRemoteProcedureCall", "gRPC")
			| ("TierHTTPProxy", "HandRolled")
			| ("TierLogger", "Standard")
			| ("TierFileSystem", "Layer2" | "Layer3")
			| ("TierFindFiles", "Layer3")
			| ("TierGlob", "JavaScript")
			| ("TierFileWatcher", "Stub")
			// Only Embedded is the silent default; FileSystem emits a cfg feature.
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
			// TierIPC is a runtime-only TS tier (read via std::env::var at runtime,
			// not compiled into Cargo features). All values are defaults from
			// Mountain's perspective - no Cargo feature needed.
			| ("TierIPC", "Mountain" | "NodeDeferred" | "Node")
			// Per-subsystem routing tiers (mod.rs reads via env!() at runtime
			// and routes to native vs. Cocoon vs. disabled).
			| ("TierTerminal", "Mountain" | "Node")
			| ("TierSCM", "Mountain" | "Node")
			| ("TierDebug", "Mountain" | "Node")
			| ("TierLanguageFeatures", "Mountain" | "Node")
			| ("TierSearch", "Mountain" | "Node")
			| ("TierOutputChannel", "Mountain" | "Node")
			| ("TierNativeHost", "Mountain" | "Node")
			| ("TierTreeView", "Mountain" | "Node")
			| ("TierStorage", "Mountain" | "Node")
			| ("TierModel", "Mountain" | "Node")
			| ("TierTasks", "Mountain" | "Node")
			| ("TierAuth", "Mountain" | "Node")
			| ("TierEncryption", "Mountain" | "Node")
			| ("TierExtensionHost", "Process" | "WebWorker" | "Disabled")
			| ("TierWebSocket", "Disabled" | "Mountain" | "Mist")
			| ("TierCommandEventBroadcast", "On" | "Off")
	)
}

/// Read a single tier key from the `.env.Land` file by walking up from
/// `CARGO_MANIFEST_DIR`. Used to keep the `tauri.conf.json` mutation in
/// sync with `PropagateTierGating` even when the process env is stale.
///
/// Returns `None` when the env file is absent or the key is not found.
fn ReadTierValueFromEnvFile(Key:&str) -> Option<String> {
	let Manifest = std::env::var("CARGO_MANIFEST_DIR").ok()?;

	let ManifestPath = std::path::PathBuf::from(Manifest);

	for Base in ManifestPath.ancestors().take(5) {
		for Candidate in [".env.Land", ".env.Land.Sample"] {
			let Full = Base.join(Candidate);

			if !Full.exists() {
				continue;
			}

			let Contents = std::fs::read_to_string(&Full).ok()?;

			for Line in Contents.lines() {
				let Trimmed = Line.trim();

				if Trimmed.is_empty() || Trimmed.starts_with('#') {
					continue;
				}

				if let Some((K, V)) = Trimmed.split_once('=') {
					if K.trim() == Key {
						return Some(V.trim().trim_matches('"').to_string());
					}
				}
			}

			// File found but key absent - stop here, don't search parents.
			return None;
		}
	}

	None
}
