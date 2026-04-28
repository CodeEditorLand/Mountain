#![allow(non_snake_case, dead_code)]

//! Resolves the Node.js binary used to spawn Cocoon.
//!
//! Ladder (first hit wins, cached in `OnceLock`):
//! `Pick` override → shipped (`Resources/Node/bin/node`) →
//! fnm → volta → asdf → nvm → homebrew → PATH `node`.
//!
//! Each step logs its outcome so the resolved source is visible in the log.

use std::{
	path::{Path, PathBuf},
	sync::OnceLock,
};

use tauri::{AppHandle, Manager, Runtime, path::BaseDirectory};

use crate::dev_log;

/// Result of a Node binary resolution attempt. Carries both the path and
/// the source so logs can distinguish shipped Node from system Node.
#[derive(Debug, Clone)]
pub struct ResolvedNode {
	pub Path:PathBuf,
	pub Source:NodeSource,
}

/// Where the Node binary came from. Ordered by preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSource {
	/// `Pick` environment variable.
	Override,
	/// Shipped with Mountain - `Resources/Node/bin/node` or dev-tree
	/// equivalent.
	Shipped,
	/// fnm's `current/bin/node`.
	Fnm,
	/// Volta's `tools/image/node/<version>/bin/node`.
	Volta,
	/// asdf's `shims/node` - resolves via `.tool-versions`.
	Asdf,
	/// nvm's `versions/node/<default>/bin/node`.
	Nvm,
	/// Homebrew - `/opt/homebrew/bin/node` (Apple Silicon) or
	/// `/usr/local/bin/node` (Intel macOS / Linuxbrew).
	Homebrew,
	/// PATH-resolved `node` - last resort fallback.
	Path,
}

impl NodeSource {
	pub fn AsLabel(self) -> &'static str {
		match self {
			Self::Override => "override",
			Self::Shipped => "shipped",
			Self::Fnm => "fnm",
			Self::Volta => "volta",
			Self::Asdf => "asdf",
			Self::Nvm => "nvm",
			Self::Homebrew => "homebrew",
			Self::Path => "path",
		}
	}
}

static RESOLVED: OnceLock<ResolvedNode> = OnceLock::new();

/// Resolve the Node binary to spawn Cocoon with. Caches the result for the
/// life of the process. If all resolution fails, returns the string `"node"`
/// so `Command::new` still tries a bare PATH lookup at spawn time - that
/// matches the legacy behaviour while logging the chain of misses.
pub fn ResolveNodeBinary<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> ResolvedNode {
	if let Some(Cached) = RESOLVED.get() {
		return Cached.clone();
	}

	let Resolved = ResolveUncached(ApplicationHandle);

	let Version = QueryNodeVersion(&Resolved.Path);
	match &Version {
		Some(Reported) => {
			dev_log!(
				"cocoon",
				"[NodeResolver] Using: {} (source={}, version={})",
				Resolved.Path.display(),
				Resolved.Source.AsLabel(),
				Reported
			);
			CheckMinMajor(Reported);
		},
		None => {
			dev_log!(
				"cocoon",
				"[NodeResolver] Using: {} (source={}, version=unknown)",
				Resolved.Path.display(),
				Resolved.Source.AsLabel()
			);
		},
	}

	// OnceLock::set is infallible after the None check above, and a race is
	// benign - both callers would resolve to the same result.
	let _ = RESOLVED.set(Resolved.clone());

	Resolved
}

/// Run `node --version` on the resolved binary and return its reported
/// version string (e.g. `v24.8.0`). Returns `None` when the binary can't be
/// spawned (bare `node` fallback under a misconfigured PATH) or when it
/// exits non-zero. Timeout isn't needed - `node --version` never blocks.
fn QueryNodeVersion(NodePath:&Path) -> Option<String> {
	let Output = std::process::Command::new(NodePath).arg("--version").output().ok()?;
	if !Output.status.success() {
		return None;
	}
	let Reported = String::from_utf8(Output.stdout).ok()?.trim().to_string();
	if Reported.is_empty() { None } else { Some(Reported) }
}

/// Emit a warning log line when the resolved Node's major version is below
/// `Require`. Does NOT fail the spawn - Cocoon's bundled code
/// mostly degrades gracefully on older engines, and operators should be
/// free to experiment on unreleased Node without a hard gate.
fn CheckMinMajor(VersionString:&str) {
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
			"warn: [NodeResolver] Node {} is below Require={}; extension host may fail to boot. Override \
			 via Pick or upgrade Node.",
			VersionString,
			Required
		);
	}
}

fn ResolveUncached<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> ResolvedNode {
	if let Some(Found) = TryOverride() {
		return Found;
	}
	if let Some(Found) = TryShipped(ApplicationHandle) {
		return Found;
	}
	if let Some(Found) = TryFnm() {
		return Found;
	}
	if let Some(Found) = TryVolta() {
		return Found;
	}
	if let Some(Found) = TryAsdf() {
		return Found;
	}
	if let Some(Found) = TryNvm() {
		return Found;
	}
	if let Some(Found) = TryHomebrew() {
		return Found;
	}

	dev_log!(
		"cocoon",
		"[NodeResolver] No specific install found; falling back to `node` on PATH"
	);

	ResolvedNode { Path:PathBuf::from("node"), Source:NodeSource::Path }
}

fn TryOverride() -> Option<ResolvedNode> {
	let Raw = std::env::var("Pick").ok()?;
	let Expanded = ExpandHome(&Raw);
	if Expanded.exists() {
		Some(ResolvedNode { Path:Expanded, Source:NodeSource::Override })
	} else {
		dev_log!(
			"cocoon",
			"warn: [NodeResolver] Pick={} does not exist; ignoring",
			Raw
		);
		None
	}
}

fn TryShipped<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> Option<ResolvedNode> {
	// Production: Tauri bundles the shipped Node under Resources/Node/bin/node
	// (or Resources/Node/node.exe on Windows).
	let RelativeToResource = if cfg!(target_os = "windows") {
		"Node/node.exe"
	} else {
		"Node/bin/node"
	};

	if let Ok(Resolved) = ApplicationHandle.path().resolve(RelativeToResource, BaseDirectory::Resource) {
		if Resolved.exists() {
			return Some(ResolvedNode { Path:Resolved, Source:NodeSource::Shipped });
		}
	}

	// Dev: executable at Target/<profile>/Mountain. Shipped Node would live
	// at Target/<profile>/Node/bin/node alongside the binary so the dev
	// build can dogfood the same resolution path as production.
	let ExecutablePath = std::env::current_exe().ok()?;
	let ExecutableDirectory = ExecutablePath.parent()?;
	let SiblingNode = ExecutableDirectory.join(RelativeToResource);
	if SiblingNode.exists() {
		return Some(ResolvedNode { Path:SiblingNode, Source:NodeSource::Shipped });
	}

	None
}

fn TryFnm() -> Option<ResolvedNode> {
	// fnm exposes `FNM_MULTISHELL_PATH` in the active shell; its `bin/node`
	// is the version pinned by the current directory's `.nvmrc` /
	// `.node-version`.
	if let Ok(Multishell) = std::env::var("FNM_MULTISHELL_PATH") {
		let Candidate = PathBuf::from(Multishell).join("bin").join(NodeExecutableName());
		if Candidate.exists() {
			return Some(ResolvedNode { Path:Candidate, Source:NodeSource::Fnm });
		}
	}

	// Fallback: `~/.local/share/fnm/current` symlink (Linux default).
	let Home = std::env::var("HOME").ok()?;
	for Relative in ["/.local/share/fnm/current/bin", "/Library/Caches/fnm_multishells/current/bin"] {
		let Candidate = PathBuf::from(&Home)
			.join(Relative.trim_start_matches('/'))
			.join(NodeExecutableName());
		if Candidate.exists() {
			return Some(ResolvedNode { Path:Candidate, Source:NodeSource::Fnm });
		}
	}
	None
}

fn TryVolta() -> Option<ResolvedNode> {
	let VoltaHome = std::env::var("VOLTA_HOME").ok().or_else(|| {
		std::env::var("HOME").ok().map(|H| PathBuf::from(H).join(".volta").to_string_lossy().into_owned())
	})?;
	// Volta's default-version symlink: <VOLTA_HOME>/tools/image/node/current/bin/node
	// but in practice Volta creates shim binaries under <VOLTA_HOME>/bin.
	let ShimCandidate = PathBuf::from(&VoltaHome).join("bin").join(NodeExecutableName());
	if ShimCandidate.exists() {
		return Some(ResolvedNode { Path:ShimCandidate, Source:NodeSource::Volta });
	}
	None
}

fn TryAsdf() -> Option<ResolvedNode> {
	let AsdfDataDir = std::env::var("ASDF_DATA_DIR").ok().or_else(|| {
		std::env::var("HOME").ok().map(|H| PathBuf::from(H).join(".asdf").to_string_lossy().into_owned())
	})?;
	// asdf shims resolve the active `.tool-versions` entry on every call.
	let ShimCandidate = PathBuf::from(&AsdfDataDir).join("shims").join(NodeExecutableName());
	if ShimCandidate.exists() {
		return Some(ResolvedNode { Path:ShimCandidate, Source:NodeSource::Asdf });
	}
	None
}

fn TryNvm() -> Option<ResolvedNode> {
	// `NVM_BIN` is set inside a shell with nvm sourced; it points directly
	// at `<nvm_dir>/versions/node/<current>/bin`.
	if let Ok(NvmBin) = std::env::var("NVM_BIN") {
		let Candidate = PathBuf::from(NvmBin).join(NodeExecutableName());
		if Candidate.exists() {
			return Some(ResolvedNode { Path:Candidate, Source:NodeSource::Nvm });
		}
	}

	// Fallback: walk `$NVM_DIR/versions/node` / `~/.nvm/versions/node` and
	// pick the lexicographically largest version (rough proxy for "latest
	// installed"). Users who want a specific version should export
	// `Pick` instead.
	let NvmDir = std::env::var("NVM_DIR").ok().or_else(|| {
		std::env::var("HOME").ok().map(|H| PathBuf::from(H).join(".nvm").to_string_lossy().into_owned())
	})?;

	let VersionsDirectory = PathBuf::from(&NvmDir).join("versions").join("node");
	let Entries = std::fs::read_dir(&VersionsDirectory).ok()?;

	let mut BestCandidate:Option<PathBuf> = None;
	for Entry in Entries.flatten() {
		let NodePath = Entry.path().join("bin").join(NodeExecutableName());
		if !NodePath.exists() {
			continue;
		}
		BestCandidate = match BestCandidate {
			Some(Existing) if Existing > NodePath => Some(Existing),
			_ => Some(NodePath),
		};
	}

	BestCandidate.map(|Path| ResolvedNode { Path, Source:NodeSource::Nvm })
}

fn TryHomebrew() -> Option<ResolvedNode> {
	for Candidate in ["/opt/homebrew/bin/node", "/usr/local/bin/node", "/home/linuxbrew/.linuxbrew/bin/node"] {
		let Path = PathBuf::from(Candidate);
		if Path.exists() {
			return Some(ResolvedNode { Path, Source:NodeSource::Homebrew });
		}
	}
	None
}

fn NodeExecutableName() -> &'static str {
	if cfg!(target_os = "windows") { "node.exe" } else { "node" }
}

fn ExpandHome(Raw:&str) -> PathBuf {
	if let Some(Stripped) = Raw.strip_prefix("~/") {
		if let Ok(Home) = std::env::var("HOME") {
			return PathBuf::from(Home).join(Stripped);
		}
	}
	PathBuf::from(Raw)
}

#[cfg(test)]
mod Tests {
	use super::*;

	#[test]
	fn NodeExecutableNameMatchesPlatform() {
		let Name = NodeExecutableName();
		if cfg!(target_os = "windows") {
			assert_eq!(Name, "node.exe");
		} else {
			assert_eq!(Name, "node");
		}
	}

	#[test]
	fn ExpandHomePreservesAbsolute() {
		let Absolute = Path::new("/usr/local/bin/node");
		assert_eq!(ExpandHome("/usr/local/bin/node"), Absolute);
	}
}
