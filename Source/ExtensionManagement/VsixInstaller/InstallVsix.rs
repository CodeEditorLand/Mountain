//! `VsixInstaller::InstallVsix`

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

/// Open `VsixPath` and Install its payload under `InstallRoot`. On success the
/// caller receives the new identifier, Install directory, and a DTO ready
/// for `Struct::AddOrUpdate`.
pub fn Fn(VsixPath:&Path, InstallRoot:&Path) -> Result<InstallOutcome, InstallError> {
	if !VsixPath.exists() {
		return Err(InstallError::SourceMissing(VsixPath.to_path_buf()));
	}

	let Facts = ReadManifestFacts(VsixPath)?;

	let InstalledAt = InstallRoot.join(format!("{}.{}-{}", Facts.Publisher, Facts.Name, Facts.Version));

	let Identifier = format!("{}.{}", Facts.Publisher, Facts.Name);

	// Idempotent reinstall: if the target directory already holds the same
	// <publisher>.<name>-<version>, skip extraction and surface the existing
	// Install as a success. Reading the on-disk manifest handles the edge
	// case where the directory was left in a half-written state by an earlier
	// crash - BuildDescription will Err, and we fall through to re-extract.
	if InstalledAt.exists() {
		if let Ok(Description) = BuildDescription(&InstalledAt) {
			// Retroactively heal exec bits on existing installs. Older
			// VSIX installs predating the magic-number / bin-path
			// promotion left native binaries (rust-analyzer's
			// `server/rust-analyzer`, openai.chatgpt's
			// `bin/<triple>/codex`, etc.) at 0o644 - the extension's
			// own `child_process.spawn(...)` then fails with EACCES
			// even though the file is intact on disk. Walk the Install
			// tree once and chmod +x anything matching the same
			// heuristic ExtractPayload uses for fresh installs.
			#[cfg(unix)]
			HealExecutableBits(&InstalledAt);

			dev_log!(
				"extensions",
				"[VsixInstaller] Reinstall no-op - '{}' v{} already present at {}",
				Identifier,
				Facts.Version,
				InstalledAt.display()
			);

			return Ok(InstallOutcome { Identifier, Version:Facts.Version, InstalledAt, Description });
		}

		// Corrupt / partial previous Install - wipe and re-extract below.
		dev_log!(
			"extensions",
			"[VsixInstaller] Existing Install at {} is unreadable - wiping and reinstalling",
			InstalledAt.display()
		);

		fs::remove_dir_all(&InstalledAt).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;
	}

	CreateParent(&InstalledAt)?;

	ExtractPayload(VsixPath, &InstalledAt)?;

	let Description = BuildDescription(&InstalledAt)?;

	dev_log!(
		"extensions",
		"[VsixInstaller] Installed '{}' v{} at {}",
		Identifier,
		Facts.Version,
		InstalledAt.display()
	);

	Ok(InstallOutcome { Identifier, Version:Facts.Version, InstalledAt, Description })
}
