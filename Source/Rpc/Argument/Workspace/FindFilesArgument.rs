

use serde::Deserialize;

// Assuming GlobPattern and FindFilesOptionsDto are accessible,
// e.g., re-exported from Rpc/Argument/Workspace/mod.rs or defined in sibling files.
use super::super::Common::GlobPattern;
use super::FindFilesOptions::FindFilesOptionsDto;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FindFilesArgument {
	// The glob pattern to include files.
	pub Include:GlobPattern,
	// Optional. The glob pattern to exclude files.
	pub Exclude:Option<GlobPattern>,
	// Optional. Additional options for the find operation.
	pub Options:Option<FindFilesOptionsDto>,
}
