//! fnm shim lookup. `FNM_MULTISHELL_PATH` (active shell) wins; otherwise
//! probe the per-OS multishell cache directories.

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::{NodeExecutableName, NodeSource, ResolvedNode};

/// Public entry point for this module.
pub fn Fn() -> Option<ResolvedNode::Struct> {
	if let Ok(Multishell) = std::env::var("FNM_MULTISHELL_PATH") {
		let Candidate = PathBuf::from(Multishell).join("bin").join(NodeExecutableName::Fn());

		if Candidate.exists() {
			return Some(ResolvedNode::Struct { Path:Candidate, Source:NodeSource::Enum::Fnm });
		}
	}

	let Home = std::env::var("HOME").ok()?;

	for Relative in ["/.local/share/fnm/current/bin", "/Library/Caches/fnm_multishells/current/bin"] {
		let Candidate = PathBuf::from(&Home)
			.join(Relative.trim_start_matches('/'))
			.join(NodeExecutableName::Fn());

		if Candidate.exists() {
			return Some(ResolvedNode::Struct { Path:Candidate, Source:NodeSource::Enum::Fnm });
		}
	}

	None
}
