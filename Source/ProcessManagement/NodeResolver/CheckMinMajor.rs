//! Warn when the resolved Node's major version is below `Require`. Does NOT
//! fail the spawn - Cocoon's bundled code mostly degrades gracefully on older
//! engines and operators should be free to experiment on unreleased Node
//! without a hard gate.

use crate::dev_log;

pub fn Fn(VersionString:&str) {

	let Trimmed = VersionString.trim_start_matches('v');

	let MajorToken = Trimmed.split('.').next().unwrap_or("");

	let Major:u32 = match MajorToken.parse() {
		Ok(Value) => Value,

		Err(_) => return,
	};

	let Required:u32 = std::env::var("Require").ok().and_then(|Raw| Raw.parse().ok()).unwrap_or(20);

	if Major < Required {
		dev_log!(
			"cocoon",

			"warn: [NodeResolver] Node {} is below Require={}; extension host may fail to boot. Override via Pick or \
			 upgrade Node.",
			VersionString,

			Required
		);
	}
}
