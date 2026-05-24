//! `VsixInstaller::UninstallExtension`

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

/// Delete the Install directory. Returns `Ok` if the path was already absent.
pub fn Fn(InstallDir:&Path) -> Result<(), InstallError> {
	if !InstallDir.exists() {
		dev_log!(
			"extensions",
			"[VsixInstaller] Uninstall skipped - {} already absent",
			InstallDir.display()
		);

		return Ok(());
	}

	fs::remove_dir_all(InstallDir).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;

	dev_log!("extensions", "[VsixInstaller] Uninstalled {}", InstallDir.display());

	Ok(())
}
