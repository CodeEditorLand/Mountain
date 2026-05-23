//! Bring up the global PostHog client and emit
//! `land:mountain:session:start`. Must be called once during boot;
//! re-entrancy is safe because the underlying `OnceLock::set` returns
//! `Err` on subsequent attempts.
//!
//! Note: posthog-rs 0.5 renamed `api_endpoint` → `host` on
//! `ClientOptionsBuilder` to match the JS/Python SDK vocabulary; keep
//! `host(...)` here.

use crate::{
	Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent, Client, Constants},
	dev_log,
};

pub async fn Fn() {
	if !CaptureAllowed::Fn() {
		return;
	}

	let Options = posthog_rs::ClientOptionsBuilder::default()
		.api_key(Constants::POSTHOG_API_KEY.to_string())
		.host(Constants::POSTHOG_HOST.to_string())
		.build()
		.expect("PostHog client options");

	let PostHogClient = posthog_rs::client(Options).await;

	let _ = Client::CLIENT.set(PostHogClient);

	dev_log!(
		"lifecycle",
		"[PostHog] Initialized (host={}, debug mode)",
		Constants::POSTHOG_HOST
	);

	CaptureEvent::Fn(
		"land:mountain:session:start",
		Some(vec![
			("pid", Box::leak(format!("{}", std::process::id()).into_boxed_str())),
			("os", std::env::consts::OS),
			("arch", std::env::consts::ARCH),
		]),
	);
}
