

use serde::Deserialize;

// Defines options for the FindFiles operation in the workspace.
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FindFilesOptionsDto {
	// Renamed from FindFilesOptions to FindFilesOptionsDto
	// Optional. The maximum number of results to return.
	#[serde(alias = "maxResults")]
	pub MaxResults:Option<usize>,
	// Optional. If true (default), respects .gitignore and similar ignore files.
	#[serde(alias = "useIgnoreFiles")]
	pub UseIgnoreFiles:Option<bool>,
	// Optional. If true (default), respects global git ignore files.
	#[serde(alias = "useGlobalIgnoreFiles")]
	pub UseGlobalIgnoreFiles:Option<bool>,
	// Optional. If true (default), respects ignore files in parent directories.
	#[serde(alias = "useParentIgnoreFiles")]
	pub UseParentIgnoreFiles:Option<bool>,
	// Optional. If true, symbolic links will be followed. Defaults to false.
	#[serde(alias = "followSymlinks")]
	pub FollowSymlinks:Option<bool>,
}
