//! Machine-stable distinct ID for the dev session. When the `Brand`
//! env var is populated, it wins - same value across every process in
//! the same dev run. Otherwise falls back to `land-dev-<USER>`.

use crate::Binary::Build::PostHogPlugin::Constants;

pub fn Fn() -> String {
	if !Constants::POSTHOG_DISTINCT_ID_SEED.is_empty() {
		return Constants::POSTHOG_DISTINCT_ID_SEED.to_string();
	}

	let User = std::env::var("USER")
		.or_else(|_| std::env::var("USERNAME"))
		.unwrap_or_else(|_| "unknown".to_string());

	format!("land-dev-{}", User)
}
