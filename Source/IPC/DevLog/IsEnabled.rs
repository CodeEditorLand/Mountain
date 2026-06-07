//! Tag-resolution helper. Reads `Trace` once into a static
//! list, then matches against four rules per call:
//!
//! 1. Explicit tag match always wins.
//! 2. `Trace=all` opens every tag.
//! 3. `Trace=short` opens every tag *except* the firehose-mute list
//!    (`SHORT_MODE_MUTED_TAGS`).
//! 4. Otherwise the tag is closed.

use std::sync::OnceLock;

static ENABLED_TAGS:OnceLock<Vec<String>> = OnceLock::new();

const SHORT_MODE_MUTED_TAGS:&[&str] = &[
	"grpc-verbose",

	"vfs-verbose",

	"fs-route",

	"tauri-invoke",

	"rpc-latency",

	"tree-latency",

	"nls",

	"fs-read",

	"preflight",

	"wsns",

	"storage-verbose",

	"config-prime",

	"cel-dispatch",

	"output-verbose",

	"command-register",

	"provider-register",

	"ext-scan-verbose",

	"channel-stub",

	"commands-verbose",

	"scheme-assets",

	"cocoon-stderr-verbose",

	"vscode-api-gap",
];

pub(super) fn EnabledTags() -> &'static Vec<String> {

	ENABLED_TAGS.get_or_init(|| {
		match std::env::var("Trace") {
			Ok(Val) => Val.split(',').map(|S| S.trim().to_lowercase()).collect(),
			Err(_) => vec![],
		}
	})
}

pub fn Fn(Tag:&str) -> bool {

	let Tags = EnabledTags();

	if Tags.is_empty() {
		return false;
	}

	let Lower = Tag.to_lowercase();

	if Tags.iter().any(|T| T == Lower.as_str()) {
		return true;
	}

	if Tags.iter().any(|T| T == "all") {
		return true;
	}

	if Tags.iter().any(|T| T == "short") {
		return !SHORT_MODE_MUTED_TAGS.iter().any(|Muted| *Muted == Lower.as_str());
	}

	false
}
