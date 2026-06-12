//! Bundled Node lookup. Production: Tauri ships Node under
//! `Resources/Node/bin/node` (`Resources/Node/node.exe` on Windows). Dev:
//! same layout next to the executable so the dev build dogfoods the same
//! resolution path.

use tauri::{AppHandle, Manager, Runtime, path::BaseDirectory};

use crate::ProcessManagement::NodeResolver::{NodeSource, ResolvedNode};

/// fn.
pub fn Fn<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> Option<ResolvedNode::Struct> {
	let RelativeToResource = if cfg!(target_os = "windows") { "Node/node.exe" } else { "Node/bin/node" };

	if let Ok(Resolved) = ApplicationHandle.path().resolve(RelativeToResource, BaseDirectory::Resource) {
		if Resolved.exists() {
			return Some(ResolvedNode::Struct { Path:Resolved, Source:NodeSource::Enum::Shipped });
		}
	}

	let ExecutablePath = std::env::current_exe().ok()?;

	let ExecutableDirectory = ExecutablePath.parent()?;

	let SiblingNode = ExecutableDirectory.join(RelativeToResource);

	if SiblingNode.exists() {
		return Some(ResolvedNode::Struct { Path:SiblingNode, Source:NodeSource::Enum::Shipped });
	}

	// Tauri externalBin sidecar layout: the binary is placed directly next to
	// the main executable as `node` (not in a `Node/bin/` subdirectory).
	// Matches `externalBin: ["Binary/node"]` in tauri.conf.json.
	let SidecarName = if cfg!(target_os = "windows") { "node.exe" } else { "node" };

	let SidecarNode = ExecutableDirectory.join(SidecarName);

	if SidecarNode.exists() {
		return Some(ResolvedNode::Struct { Path:SidecarNode, Source:NodeSource::Enum::Shipped });
	}

	None
}
