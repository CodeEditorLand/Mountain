use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Line = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = format!("{}\n", Line);

	super::OutputAppend::RecordAppend(Env, &Channel, &Text);

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
