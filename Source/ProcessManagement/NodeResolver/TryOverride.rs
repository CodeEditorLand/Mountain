//! `Pick` env-var override. Returns `Some` when the path exists, logs a
//! warning when it doesn't, and returns `None` otherwise.

use crate::{
	ProcessManagement::NodeResolver::{ExpandHome, NodeSource, ResolvedNode},
	dev_log,
};

pub fn Fn() -> Option<ResolvedNode::Struct> {
	let Raw = std::env::var("Pick").ok()?;

	let Expanded = ExpandHome::Fn(&Raw);

	if Expanded.exists() {
		Some(ResolvedNode::Struct { Path:Expanded, Source:NodeSource::Enum::Override })
	} else {
		dev_log!("cocoon", "warn: [NodeResolver] Pick={} does not exist; ignoring", Raw);

		None
	}
}
