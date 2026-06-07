//! Volta shim lookup. `VOLTA_HOME` wins; otherwise `~/.volta`. Volta installs
//! shim binaries under `<VOLTA_HOME>/bin`.

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::{NodeExecutableName, NodeSource, ResolvedNode};

pub fn Fn() -> Option<ResolvedNode::Struct> {

	let VoltaHome = std::env::var("VOLTA_HOME").ok().or_else(|| {
		std::env::var("HOME")
			.ok()
			.map(|H| PathBuf::from(H).join(".volta").to_string_lossy().into_owned())
	})?;

	let ShimCandidate = PathBuf::from(&VoltaHome).join("bin").join(NodeExecutableName::Fn());

	if ShimCandidate.exists() {
		return Some(ResolvedNode::Struct { Path:ShimCandidate, Source:NodeSource::Enum::Volta });
	}

	None
}
