//! # Shim — Gate\n//!
//! Reads `TierShim` at compile time via `env!("TierShim")` (baked by
//! `build.rs → EmitTierDefaults`) and exports boolean gate functions.
//!
//! ## Shim Levels
//!
//! | Level | Meaning |
//! |-------|---------|
//! | `None` | All shim code compiled out — zero overhead (default) |
//! | `Proxy` | 🔵 Audit-only: observe service resolution, no redirect |
//! | `Replace` | 🔵 Replace individual services with Land shims |
//! | `Own` | 🟠 Land owns InstantiationService container |
//! | `Preempt` | 🟠 Land controls BrowserMain.open() entirely |

/// Resolved at compile time — `build.rs` emits `cargo:rustc-env=TierShim=...`
const TIER_SHIM: &str = env!("TierShim");

/// Master gate: true when ANY shim functionality is active.
pub fn is_enabled() -> bool {
	TIER_SHIM != "None"
}

/// 🔵 Audit-only: observe, don't change anything.
pub fn is_proxy() -> bool {
	TIER_SHIM == "Proxy"
}

/// 🔵 Service replacement active (Replace or deeper).
pub fn is_replace_or_deeper() -> bool {
	matches!(TIER_SHIM, "Replace" | "Own" | "Preempt")
}

/// 🟠 Full container ownership active (Own or Preempt).
pub fn is_own_or_preempt() -> bool {
	matches!(TIER_SHIM, "Own" | "Preempt")
}

/// 🟠 Nuclear option: Land controls BrowserMain.open().
pub fn is_preempt() -> bool {
	TIER_SHIM == "Preempt"
}

/// The current shim level as a string.
pub fn current_level() -> &'static str {
	TIER_SHIM
}
