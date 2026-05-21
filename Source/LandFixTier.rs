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
	let IPC = std::env::var("TierIPC").unwrap_or_else(|_| "Mountain".into());

	dev_log!("lifecycle", "[LandFix:Tier] Runtime: IPC={}", IPC);
}
