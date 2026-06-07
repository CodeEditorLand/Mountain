//! Top-level shape of a `.code-workspace` JSON document. Private to the
//! parser; exposed only as `pub(super)` for the sibling
//! `ParseWorkspaceFile::Fn` to deserialise into.

use serde::Deserialize;

use crate::Workspace::WorkspaceFileService::WorkspaceFolderEntry;

#[derive(Deserialize, Debug)]
pub(super) struct Struct {

	pub(super) folders:Vec<WorkspaceFolderEntry::Struct>,

	// `.code-workspace` may also contain `settings`, `extensions`, etc. -
	// not yet consumed by Mountain.
}
