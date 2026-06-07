//! Hydrate the running process's environment from the compile-baked
//! `Constants` so child processes spawned later (Cocoon Node, Sky
//! webview) see the same telemetry config Mountain itself was built
//! with - even when the user runs the bare binary without sourcing
//! `.env.Land.PostHog`.
//!
//! Idempotent: skips any var that's already set so a CI / dev-shell
//! override beats the build-time default. Mountain's
//! `ProcessManagement::CocoonManagement::LandEnvAllowList` then
//! forwards each value into Cocoon via `Command.envs()`. Sky reads
//! the same values via `import.meta.env` substitution at Vite/Astro
//! build time.
//!
//! Release builds skip the hydration: `cfg!(debug_assertions)` is
//! `false`, so the body short-circuits and no telemetry env leaks
//! into a packaged production binary.

use crate::{Binary::Build::PostHogPlugin::Constants, dev_log};

pub fn Fn() {

	if !cfg!(debug_assertions) {
		return;
	}

	for (Key, Value) in [
		("Authorize", Constants::POSTHOG_API_KEY),

		("Beam", Constants::POSTHOG_HOST),

		("Report", Constants::POSTHOG_ENABLED),

		("Brand", Constants::POSTHOG_DISTINCT_ID_SEED),

		("Pipe", Constants::OTLP_ENDPOINT),

		("Emit", Constants::OTLP_ENABLED),

		("Capture", Constants::TELEMETRY_CAPTURE),
	] {
		if Value.is_empty() {
			continue;
		}

		// Already-set values win; this hydration is a fallback for the
		// "user runs bare binary" path.
		if std::env::var_os(Key).is_some() {
			continue;
		}

		// SAFETY: set_var on a single-threaded boot path before any
		// other thread spawns. Mountain calls this from the early boot
		// section of Binary::Main::Entry::Fn before tokio / scheduler
		// init.
		unsafe { std::env::set_var(Key, Value) };
	}

	dev_log!(
		"lifecycle",

		"[PostHog] Hydrated runtime env from baked Constants (Authorize={}, Beam={}, Capture={}, Emit={})",

		if Constants::POSTHOG_API_KEY.is_empty() { "<unset>" } else { "<set>" },

		Constants::POSTHOG_HOST,

		Constants::TELEMETRY_CAPTURE,

		Constants::OTLP_ENABLED,
	);
}
