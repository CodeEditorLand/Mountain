//! # Shim — SwallowMap (Rust)
//!
//! Pattern-matching decision engine for the Land Shim at the Rust/IPC level.
//! Given a method name (e.g., "statusbar:set"), decides whether to:
//!
//! - **Swallow**: Land handles it, VS Code never sees it
//! - **Passthrough**: VS Code handles it normally
//! - **Mixed**: Both handle it; Land gatekeeps the response
//! - **Discard**: Silently dropped (e.g., Microsoft telemetry)
//!
//! This is the Rust counterpart to `Wind/Source/Shim/SwallowMap.ts`.
//! Rules are checked in insertion order — first match wins.

/// What to do with an intercepted method.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwallowAction {
	Swallow,
	Passthrough,
	Mixed,
	Discard,
}

/// Where to route a swallowed method.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RedirectTarget {
	Wind,
	Cocoon,
	Mountain,
	Output,
	Sky,
	None,
}

struct Rule {
	pattern: &'static str,
	action: SwallowAction,
	target: RedirectTarget,
}

/// Built-in rule table. Rules are checked in order; first match wins.
/// No match defaults to `Passthrough → None`.
static RULES: &[Rule] = &[
	// ── 🔵 Status Bar ──
	Rule { pattern: "statusbar:",        action: SwallowAction::Swallow,     target: RedirectTarget::Wind },
	Rule { pattern: "$setStatusBarMessage", action: SwallowAction::Swallow,  target: RedirectTarget::Wind },

	// ── 🔵 SCM ──
	Rule { pattern: "scm:",              action: SwallowAction::Swallow,     target: RedirectTarget::Cocoon },
	Rule { pattern: "$scm:",             action: SwallowAction::Swallow,     target: RedirectTarget::Cocoon },

	// ── 🔵 Search ──
	Rule { pattern: "search:",           action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },

	// ── 🔵 Terminal ──
	Rule { pattern: "terminal:",         action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },

	// ── 🔵 Output ──
	Rule { pattern: "output:",           action: SwallowAction::Swallow,     target: RedirectTarget::Output },

	// ── 🔵 File System ──
	Rule { pattern: "file:read",         action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },
	Rule { pattern: "file:write",        action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },
	Rule { pattern: "file:stat",         action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },
	Rule { pattern: "file:delete",       action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },
	Rule { pattern: "file:mkdir",        action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },
	Rule { pattern: "file:readdir",      action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },

	// ── 🔵 Notifications ──
	Rule { pattern: "notification:",     action: SwallowAction::Swallow,     target: RedirectTarget::Wind },

	// ── 🔵 Dialogs ──
	Rule { pattern: "dialog:",           action: SwallowAction::Swallow,     target: RedirectTarget::Mountain },

	// ── 🔵 Quick Input ──
	Rule { pattern: "quickInput:",       action: SwallowAction::Swallow,     target: RedirectTarget::Wind },

	// ── 🔵 Keybindings ──
	Rule { pattern: "keybinding:",       action: SwallowAction::Swallow,     target: RedirectTarget::Wind },

	// ── 🔵 Themes ──
	Rule { pattern: "theme:",            action: SwallowAction::Swallow,     target: RedirectTarget::Wind },

	// ── 🔵 Configuration ──
	Rule { pattern: "configuration:",    action: SwallowAction::Swallow,     target: RedirectTarget::Wind },

	// ── 🔵 Telemetry — DISCARD ──
	Rule { pattern: "telemetry:",        action: SwallowAction::Discard,     target: RedirectTarget::None },

	// ── 🔵 Extension Gallery ──
	Rule { pattern: "extensionsGallery:", action: SwallowAction::Swallow,    target: RedirectTarget::Wind },
];

/// Decide what to do with a given method name.
pub fn decide(method: &str) -> (SwallowAction, RedirectTarget) {
	for rule in RULES {
		if method.starts_with(rule.pattern) {
			return (rule.action, rule.target);
		}
	}
	(SwallowAction::Passthrough, RedirectTarget::None)
}

/// Quick check: should this method be swallowed (or discarded)?
pub fn should_swallow(method: &str) -> bool {
	let (action, _) = decide(method);
	matches!(action, SwallowAction::Swallow | SwallowAction::Discard)
}

/// Quick check: should this method pass through to VS Code?
pub fn should_passthrough(method: &str) -> bool {
	matches!(decide(method).0, SwallowAction::Passthrough)
}

/// Get the redirect target for a method.
pub fn redirect_target(method: &str) -> RedirectTarget {
	let (action, target) = decide(method);
	if matches!(action, SwallowAction::Swallow | SwallowAction::Mixed) {
		target
	} else {
		RedirectTarget::None
	}
}
