//! # LandFixTier
//!
//! Emits a single ISO-timestamped boot banner listing the compiled-in value of
//! every tier variable. Because all `env!("Tier…")` calls are resolved by
//! `build.rs::PropagateTierGating` at compile time, the banner always reflects
//! the exact configuration baked into *this* binary - not whatever the host
//! environment happens to export at runtime.
//!
//! ## Design Rationale
//!
//! Three distinct audiences depend on this banner:
//!
//! | Audience | Why it matters |
//! |---|---|
//! | Log readers (humans) | A pasted session log must show at-a-glance which tier was active when the problem occurred. |
//! | Regression triage | Confirms the binary on disk was built from the same `.env.Land` that shipped. |
//! | Cross-element agreement | Pairs with Cocoon's `[LandFix:Tier] Cocoon tier set resolved:` and Sky's `[LandFix:Tier] Sky tier set:`. A mismatch between any two signals configuration drift as the root cause. |
//!
//! ## Call Site
//!
//! `LogResolvedTiers()` is called unconditionally from
//! `Binary::Main::Entry::Fn` before the Tokio runtime begins spawning tasks.
//! `dev_log!` is synchronous, so the banner is guaranteed to land in the log
//! before any extension code runs.
//!
//! Runtime overhead is zero - all `env!(...)` invocations become string
//! literals at compile time and are inlined into a single write call.
//!
//! ## References
//!
//! See `Documentation/GitHub/Workflow/TierGatedImplementationSelection.md` for
//! the end-to-end tier-gating workflow and the matching call sites in Cocoon,
//! Wind, and Sky. Every capability listed in the boot banner maps to one or
//! more `// Tier:<Capability>:<Value>` comments at its dispatch site.

use crate::dev_log;

/// Emits one ISO-timestamped line at boot listing the compiled-in value of all
/// 17 build-baked tier variables + 1 runtime tier. Call once from
/// `Binary::Main::Entry::Fn`, after the logging infrastructure is ready
/// and before the Tokio runtime spawns any tasks.
pub fn LogResolvedTiers() {
	// Build-baked tiers use env!() - values are baked from .env.Land at build time.
	dev_log!(
		"lifecycle",
		"[LandFix:Tier] Mountain tiers: RemoteProcedureCall={} HTTPProxy={} Logger={} FileSystem={} FindFiles={} \
		 Glob={} FileWatcher={} SchemeAssets={} Configuration={} Diagnostics={} Clipboard={} OpenExternal={} \
		 DocumentMirror={} ExtensionActivation={} ExtensionScan={} ModuleCache={} Telemetry={}",
		env!("TierRemoteProcedureCall"),
		env!("TierHTTPProxy"),
		env!("TierLogger"),
		env!("TierFileSystem"),
		env!("TierFindFiles"),
		env!("TierGlob"),
		env!("TierFileWatcher"),
		env!("TierSchemeAssets"),
		env!("TierConfiguration"),
		env!("TierDiagnostics"),
		env!("TierClipboard"),
		env!("TierOpenExternal"),
		env!("TierDocumentMirror"),
		env!("TierExtensionActivation"),
		env!("TierExtensionScan"),
		env!("TierModuleCache"),
		env!("TierTelemetry"),
	);

	// Runtime-only tiers use std::env::var - readable without a rebuild.
	// `env!()` baked the compile-time default; a shell `export TierX=Node`
	// at launch overrides without requiring a new binary. mod.rs reads via
	// the same fallback chain.
	let IPC = std::env::var("TierIPC").unwrap_or_else(|_| env!("TierIPC", "Mountain").to_string());

	let Terminal = std::env::var("TierTerminal").unwrap_or_else(|_| env!("TierTerminal", "Mountain").to_string());

	let SCM = std::env::var("TierSCM").unwrap_or_else(|_| env!("TierSCM", "Mountain").to_string());

	let Debug = std::env::var("TierDebug").unwrap_or_else(|_| env!("TierDebug", "Mountain").to_string());

	let LanguageFeatures =
		std::env::var("TierLanguageFeatures").unwrap_or_else(|_| env!("TierLanguageFeatures", "Mountain").to_string());

	let Search = std::env::var("TierSearch").unwrap_or_else(|_| env!("TierSearch", "Mountain").to_string());

	let OutputChannel =
		std::env::var("TierOutputChannel").unwrap_or_else(|_| env!("TierOutputChannel", "Mountain").to_string());

	let NativeHost = std::env::var("TierNativeHost").unwrap_or_else(|_| env!("TierNativeHost", "Mountain").to_string());

	let TreeView = std::env::var("TierTreeView").unwrap_or_else(|_| env!("TierTreeView", "Mountain").to_string());

	let Storage = std::env::var("TierStorage").unwrap_or_else(|_| env!("TierStorage", "Mountain").to_string());

	let Model = std::env::var("TierModel").unwrap_or_else(|_| env!("TierModel", "Mountain").to_string());

	let Tasks = std::env::var("TierTasks").unwrap_or_else(|_| env!("TierTasks", "Node").to_string());

	let Auth = std::env::var("TierAuth").unwrap_or_else(|_| env!("TierAuth", "Node").to_string());

	let Encryption = std::env::var("TierEncryption").unwrap_or_else(|_| env!("TierEncryption", "Mountain").to_string());

	let ExtensionHost =
		std::env::var("TierExtensionHost").unwrap_or_else(|_| env!("TierExtensionHost", "Process").to_string());

	let WebSocket = std::env::var("TierWebSocket").unwrap_or_else(|_| env!("TierWebSocket", "Disabled").to_string());

	let CommandEventBroadcast = std::env::var("TierCommandEventBroadcast")
		.unwrap_or_else(|_| env!("TierCommandEventBroadcast", "Off").to_string());

	dev_log!(
		"lifecycle",
		"[LandFix:Tier] Runtime: IPC={} Terminal={} SCM={} Debug={} LanguageFeatures={} Search={} OutputChannel={} \
		 NativeHost={} TreeView={} Storage={} Model={} Tasks={} Auth={} Encryption={} ExtensionHost={} WebSocket={} \
		 CommandEventBroadcast={}",
		IPC,
		Terminal,
		SCM,
		Debug,
		LanguageFeatures,
		Search,
		OutputChannel,
		NativeHost,
		TreeView,
		Storage,
		Model,
		Tasks,
		Auth,
		Encryption,
		ExtensionHost,
		WebSocket,
		CommandEventBroadcast,
	);
}
