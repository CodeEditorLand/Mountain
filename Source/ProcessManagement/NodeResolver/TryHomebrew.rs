//! Homebrew lookup. Apple Silicon, Intel macOS, and Linuxbrew probe paths.

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::{NodeSource, ResolvedNode};

/// Public entry point for this module.
pub fn Fn() -> Option<ResolvedNode::Struct> {
	for Candidate in [
		"/opt/homebrew/bin/node",
		"/usr/local/bin/node",
		"/home/linuxbrew/.linuxbrew/bin/node",
	] {
		let Path = PathBuf::from(Candidate);

		if Path.exists() {
			return Some(ResolvedNode::Struct { Path, Source:NodeSource::Enum::Homebrew });
		}
	}

	None
}
