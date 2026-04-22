//! # LandFixTier - Mountain's runtime tier banner
//!
//! Emits one ISO-timestamped line at boot listing every tier variable's
//! compiled value. Because the values are resolved at compile time via
//! `env!("Tier…")` (populated by `build.rs::PropagateTierGating`), the
//! banner is always correct for *this particular binary*, not "whatever
//! is in the env right now".
//!
//! ## Why this shape
//!
//! Three distinct audiences need the banner:
//!
//! | Audience                | Reason                                                            |
//! | ----------------------- | ----------------------------------------------------------------- |
//! | Log readers (humans)    | A pasted session log must show at-a-glance which tier was active. |
//! | Regression triage       | Confirms the binary on disk matches the `.env.Land` that shipped. |
//! | Cross-Element agreement | Pairs with Cocoon's `[LandFix:Tier] Cocoon tier set resolved:` and Sky's `[LandFix:Tier] Sky tier set:` - if Mountain disagrees with either, configuration drift is the root cause. |
//!
//! ## Call site
//!
//! `LogResolvedTiers()` is called unconditionally from `Binary/Main/Entry::Fn`
//! before the Tokio runtime starts spawning tasks, because `dev_log!` is
//! synchronous and the banner should land before any extension code runs.
//!
//! Runtime overhead is zero - all `env!(...)` invocations resolve to string
//! literals at compile time and Rust's `println!`/`dev_log!` codegen inlines
//! the format arguments into a single write call.
//!
//! ## References
//!
//! See `Documentation/GitHub/Workflow/TierGatedImplementationSelection.md`
//! (rendered at
//! <https://github.com/CodeEditorLand/Land/tree/Current/Documentation/GitHub/Workflow/TierGatedImplementationSelection.md>)
//! for the end-to-end workflow and the matching call sites in Cocoon, Wind
//! and Sky. Every capability listed in the banner below maps to one or more
//! tier-marker comments of the form `// Tier:<Capability>:<Value>` at the
//! corresponding dispatch site.

use crate::dev_log;

/// Emits one line at boot listing every tier variable's compiled value.
/// Call once, from `tauri::Builder`'s setup hook, after the logging
/// infrastructure is ready.
pub fn LogResolvedTiers() {
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
}
