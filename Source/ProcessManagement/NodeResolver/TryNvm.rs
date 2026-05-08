#![allow(non_snake_case)]

//! nvm lookup. `NVM_BIN` wins (set inside an nvm-sourced shell). Fallback
//! walks `$NVM_DIR/versions/node` and picks the lexicographically largest
//! version (rough proxy for "latest installed").

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::{NodeExecutableName, NodeSource, ResolvedNode};

pub fn Fn() -> Option<ResolvedNode::Struct> {
	if let Ok(NvmBin) = std::env::var("NVM_BIN") {
		let Candidate = PathBuf::from(NvmBin).join(NodeExecutableName::Fn());

		if Candidate.exists() {
			return Some(ResolvedNode::Struct { Path:Candidate, Source:NodeSource::Enum::Nvm });
		}
	}

	let NvmDir = std::env::var("NVM_DIR").ok().or_else(|| {
		std::env::var("HOME")
			.ok()
			.map(|H| PathBuf::from(H).join(".nvm").to_string_lossy().into_owned())
	})?;

	let VersionsDirectory = PathBuf::from(&NvmDir).join("versions").join("node");

	let Entries = std::fs::read_dir(&VersionsDirectory).ok()?;

	let mut BestCandidate:Option<PathBuf> = None;

	for Entry in Entries.flatten() {
		let NodePath = Entry.path().join("bin").join(NodeExecutableName::Fn());

		if !NodePath.exists() {
			continue;
		}

		BestCandidate = match BestCandidate {
			Some(Existing) if Existing > NodePath => Some(Existing),

			_ => Some(NodePath),
		};
	}

	BestCandidate.map(|Path| ResolvedNode::Struct { Path, Source:NodeSource::Enum::Nvm })
}
