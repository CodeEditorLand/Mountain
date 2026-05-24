//! `VsixInstaller::ReadFullManifest`

use std::{
	fs::{self, File},
	io::{self, Read},
	path::{Path, PathBuf},
};
use serde_json::Value;
use zip::ZipArchive;
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

const MANIFEST_ENTRY:&str = "extension/package.json";
const PAYLOAD_PREFIX:&str = "extension/";

/// Read the full `extension/package.json` from a `.vsix` without extracting
/// the archive to disk. Used by the IPC `extensions:getManifest` handler so
/// the "Install from VSIX…" preview dialog and drag-and-drop flow can inspect
/// a manifest before the user confirms installation.
///
/// The returned value is the raw parsed JSON (`serde_json::Value`) - callers
/// can project it into VS Code's `IExtensionManifest` shape. No NLS bundle
/// resolution is performed here (the renderer only needs publisher/name/
/// version/displayName for the preview UI, and NLS keys would require
/// unpacking `package.nls.json` from the archive too).
pub fn Fn(VsixPath:&Path) -> Result<Value, InstallError> {
	let Archive = File::open(VsixPath).map_err(|Error| InstallError::ArchiveRead(Error.to_string()))?;

	let mut Archive = ZipArchive::new(Archive).map_err(|Error| InstallError::ArchiveRead(Error.to_string()))?;

	let mut Entry = Archive
		.by_name(MANIFEST_ENTRY)
		.map_err(|Error| InstallError::ManifestMissing(Error.to_string()))?;

	let mut Raw = String::new();

	Entry
		.read_to_string(&mut Raw)
		.map_err(|Error| InstallError::ManifestMissing(Error.to_string()))?;

	serde_json::from_str(&Raw).map_err(|Error| InstallError::ManifestMissing(Error.to_string()))
}
