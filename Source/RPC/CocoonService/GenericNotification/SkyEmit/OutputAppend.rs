use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	RecordAppend(Env, &Channel, &Text);

	// Coalesce per-channel: the Vine-side buffer flushes one
	// `sky://output/append` carrying the joined text (`{channel, value}`,
	// which Sky's handler reads alongside `text`) instead of one
	// WKWebView IPC event per appended line.
	if crate::Vine::Server::Notification::OutputChannelCoalesce::TryEnqueue(
		&Env.ApplicationHandle,
		Channel.clone(),
		Text.clone(),
	) {
		return;
	}

	let _ = Env
		.ApplicationHandle
		.emit_to("main", "sky://output/append", json!({ "channel": Channel, "text": Text }));
}

/// Retains appended text in `Feature.OutputChannels` so
/// `sky:replay-events` can rebuild the Output panel after a late
/// SkyBridge boot. The DTO's `Append` enforces the per-channel byte cap.
pub(super) fn RecordAppend(Env:&MountainEnvironment, Channel:&str, Text:&str) {
	if Channel.is_empty() || Text.is_empty() {
		return;
	}

	let mut Channels = Env.ApplicationState.Feature.OutputChannels.OutputChannels.lock();

	let Entry = Channels.entry(Channel.to_string()).or_default();

	let _ = Entry.Append(Text);
}
