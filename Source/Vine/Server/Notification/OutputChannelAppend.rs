#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.append` notification.
//! Twin of `output.append`; see `OutputCreate.rs` for the duplicate-wire
//! rationale.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/append", Parameter);
	// Per-append fire - `roo-cline`, `TypeScript`, `dart-code` all stream
	// stdout into their output channels which fires 200+ appends per
	// boot. The Sky-side consumer already sees the data via
	// `sky://output/append`; the tag line here adds no signal beyond
	// volume for THOSE channels. Route to `output-verbose`.
	//
	// Exception: vscode.git's "Git" channel logs activation flow at
	// `[Model][doInitialScan]`, `[main] Using git`, `[main] Failed to create
	// model` etc. - these are critical for diagnosing the F6 silent-bail
	// (vscode.git activates ok but never reaches createSourceControl).
	// Surface those at the `grpc` tag so they appear in `LAND_DEV_LOG=short`
	// runs without forcing the user to enable `output-verbose` and drown
	// in TypeScript / dart-code / roo-cline noise.
	let ChannelName = Parameter
		.get("channel")
		.or_else(|| Parameter.get("name"))
		.and_then(Value::as_str)
		.unwrap_or("?");
	// Char-aware truncation. Slicing a `&str` at `&S[..200]` panics when
	// byte 200 lands inside a multi-byte UTF-8 codepoint (vscode.git's
	// progress messages contain `•` which is 3 bytes; if the message is
	// >200 bytes and the bullet sits across the boundary, the slice
	// crashes the tokio worker - observed live during SCM viewlet open).
	// Walk char boundaries instead so the cut always lands between codepoints.
	let TruncatedValue = Parameter
		.get("value")
		.and_then(Value::as_str)
		.map(|S| {
			if S.len() > 200 {
				let CutAt = S
					.char_indices()
					.map(|(Index, _)| Index)
					.take_while(|Index| *Index <= 200)
					.last()
					.unwrap_or(0);
				format!("{}…", &S[..CutAt])
			} else {
				S.to_string()
			}
		})
		.unwrap_or_else(|| "<no-value>".to_string());
	if ChannelName.eq_ignore_ascii_case("git")
		|| ChannelName.eq_ignore_ascii_case("source control")
		|| ChannelName.eq_ignore_ascii_case("scm")
	{
		dev_log!(
			"grpc",
			"[OutputChannel:{}] {}",
			ChannelName,
			TruncatedValue.trim_end_matches('\n')
		);
	} else {
		dev_log!(
			"output-verbose",
			"[OutputChannel] append channel={}",
			ChannelName
		);
	}
}
