//! asdf shim lookup. The shim resolves the active `.tool-versions` entry on
//! every call.

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::{NodeExecutableName, NodeSource, ResolvedNode};

/// Public entry point for this module.
pub fn Fn() -> Option<ResolvedNode::Struct> {
	let AsdfDataDir = std::env::var("ASDF_DATA_DIR").ok().or_else(|| {
		std::env::var("HOME")
			.ok()
			.map(|H| PathBuf::from(H).join(".asdf").to_string_lossy().into_owned())
	})?;

	let ShimCandidate = PathBuf::from(&AsdfDataDir).join("shims").join(NodeExecutableName::Fn());

	if ShimCandidate.exists() {
		return Some(ResolvedNode::Struct { Path:ShimCandidate, Source:NodeSource::Enum::Asdf });
	}

	None
}
