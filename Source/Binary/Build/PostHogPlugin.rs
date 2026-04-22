//! # PostHog Plugin Module
//!
//! Debug-only PostHog analytics integration for Mountain.
//! Captures lifecycle events, IPC commands, errors, and performance metrics.
//! Disabled in release builds (compile-time gated).

use std::sync::OnceLock;

use crate::dev_log;

/// PostHog project token. Source of truth: `.env.Land.PostHog` LAND_POSTHOG_KEY;
/// `build.rs` bakes the value via `cargo:rustc-env` so `env!` at compile
/// time always resolves, even on a clean checkout.
const POSTHOG_API_KEY:&str = env!("LAND_POSTHOG_KEY");

/// PostHog region host (default EU Cloud; operators override via
/// `.env.Land.PostHog` LAND_POSTHOG_HOST).
const POSTHOG_HOST:&str = env!("LAND_POSTHOG_HOST");

/// Per-tier enable flag baked from `.env.Land.PostHog`. Cheap early-exit in
/// every capture path without forking the binary per env value.
const POSTHOG_ENABLED:&str = env!("LAND_POSTHOG_MOUNTAIN_ENABLED");

/// Optional pinned distinct-id seed (empty string → auto-generate per
/// process). Useful for CI runs where correlating events across restarts
/// matters more than per-dev isolation.
const POSTHOG_DISTINCT_ID_SEED:&str = env!("LAND_POSTHOG_DISTINCT_ID");

/// Global PostHog client instance.
static CLIENT:OnceLock<posthog_rs::Client> = OnceLock::new();

/// Machine-stable distinct ID for the dev session. When LAND_POSTHOG_DISTINCT_ID
/// is set, it wins - same value across every process in the same dev run.
fn DistinctId() -> String {
	if !POSTHOG_DISTINCT_ID_SEED.is_empty() {
		return POSTHOG_DISTINCT_ID_SEED.to_string();
	}
	let User = std::env::var("USER")
		.or_else(|_| std::env::var("USERNAME"))
		.unwrap_or_else(|_| "unknown".to_string());
	format!("land-dev-{}", User)
}

/// Whether the Mountain tier should capture at all. Combines compile-time
/// debug gate with the `.env.Land.PostHog` enable switch.
fn CaptureAllowed() -> bool {
	if !cfg!(debug_assertions) {
		return false;
	}
	!matches!(POSTHOG_ENABLED, "false" | "0" | "off")
}

/// Initialize the PostHog client. Call once during app setup.
/// No-op in release builds or when LAND_POSTHOG_MOUNTAIN_ENABLED=false.
pub async fn Initialize() {
	if !CaptureAllowed() {
		return;
	}

	// posthog-rs 0.5 renamed `api_endpoint` → `host` on `ClientOptionsBuilder`
	// to match the JS/Python SDK vocabulary. Behaviour unchanged: the value is
	// the base URL the ingestion client POSTs to. Pass `String`-typed so the
	// `#[builder(setter(into, strip_option))]` attribute on `host` wraps it in
	// `Some(...)` without another hop.
	let Options = posthog_rs::ClientOptionsBuilder::default()
		.api_key(POSTHOG_API_KEY.to_string())
		.host(POSTHOG_HOST.to_string())
		.build()
		.expect("PostHog client options");

	let PostHogClient = posthog_rs::client(Options).await;
	let _ = CLIENT.set(PostHogClient);
	dev_log!("lifecycle", "[PostHog] Initialized (host={}, debug mode)", POSTHOG_HOST);
	CaptureEvent("mountain:session:start", None);
}

/// Capture a named event with optional properties.
pub fn CaptureEvent(EventName:&str, Properties:Option<Vec<(&str, &str)>>) {
	if !CaptureAllowed() {
		return;
	}

	let Some(Client) = CLIENT.get() else { return };

	let mut Event = posthog_rs::Event::new(EventName, &DistinctId());

	let _ = Event.insert_prop("$app", "land-editor");
	let _ = Event.insert_prop("$app_version", "0.0.1");
	let _ = Event.insert_prop("$build_mode", "debug");
	let _ = Event.insert_prop("$component", "mountain");

	if let Some(Props) = Properties {
		for (Key, Value) in Props {
			let _ = Event.insert_prop(Key, Value);
		}
	}

	let _ = Client.capture(Event);
}

/// Capture an error event.
pub fn CaptureError(Tag:&str, Message:&str) {
	if !CaptureAllowed() {
		return;
	}

	CaptureEvent("mountain:error", Some(vec![("error_tag", Tag), ("error_message", Message)]));
}

/// Capture an IPC command invocation.
pub fn CaptureIPC(Method:&str) {
	if !CaptureAllowed() {
		return;
	}

	CaptureEvent("mountain:ipc:invoke", Some(vec![("method", Method)]));
}
