//! # PostHog Plugin Module
//!
//! Debug-only PostHog analytics integration for Mountain.
//! Captures lifecycle events, IPC commands, errors, and performance metrics.
//! Disabled in release builds (compile-time gated).

use std::sync::OnceLock;

use log::debug;

/// PostHog EU Cloud project token (debug builds only).
const POSTHOG_API_KEY:&str = "phc_mCwHy7LgvbnEqh6a2DyMiLUJcaZvmmj7JNmmpQzvr7mA";

/// PostHog EU Cloud host.
const POSTHOG_HOST:&str = "https://eu.i.posthog.com";

/// Global PostHog client instance.
static CLIENT:OnceLock<posthog_rs::Client> = OnceLock::new();

/// Machine-stable distinct ID for the dev session.
fn DistinctId() -> String {
	let User = std::env::var("USER")
		.or_else(|_| std::env::var("USERNAME"))
		.unwrap_or_else(|_| "unknown".to_string());
	format!("land-dev-{}", User)
}

/// Initialize the PostHog client. Call once during app setup.
/// No-op in release builds.
pub async fn Initialize() {
	if !cfg!(debug_assertions) {
		return;
	}

	let Options = posthog_rs::ClientOptionsBuilder::default()
		.api_key(POSTHOG_API_KEY.to_string())
		.api_endpoint(POSTHOG_HOST.to_string())
		.build()
		.expect("PostHog client options");

	let PostHogClient = posthog_rs::client(Options).await;
	let _ = CLIENT.set(PostHogClient);
	debug!("[PostHog] Initialized (EU Cloud, debug mode)");
	CaptureEvent("mountain:session:start", None);
}

/// Capture a named event with optional properties.
pub fn CaptureEvent(EventName:&str, Properties:Option<Vec<(&str, &str)>>) {
	if !cfg!(debug_assertions) {
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
	if !cfg!(debug_assertions) {
		return;
	}

	CaptureEvent("mountain:error", Some(vec![("error_tag", Tag), ("error_message", Message)]));
}

/// Capture an IPC command invocation.
pub fn CaptureIPC(Method:&str) {
	if !cfg!(debug_assertions) {
		return;
	}

	CaptureEvent("mountain:ipc:invoke", Some(vec![("method", Method)]));
}
