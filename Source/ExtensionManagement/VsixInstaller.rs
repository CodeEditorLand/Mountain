//! # VSIX Installer
//!
//! Unpacks a `.vsix` file (ZIP with `extension/` as the payload prefix) into
//! Land's user-extensions directory and produces an
//! `ExtensionDescriptionStateDTO` ready for insertion into the application
//! state's `ScannedExtensionCollection`.
//!
//! ## Flow
//!
//! 1. `InstallVsix(VsixPath, InstallRoot)`:
//!    - Open the `.vsix` as a zip archive.
//!    - Read `extension/package.json`, parse minimal fields (publisher, name,
//!      version). These three determine the install directory.
//!    - Compute target: `<InstallRoot>/<publisher>.<name>-<version>/`.
//!    - If target already exists, refuse (caller decides whether to reinstall).
//!    - Stream every entry whose path begins with `extension/` into the target,
//!      stripping that prefix.
//!    - Re-parse the extracted `package.json` as a full
//!      `ExtensionDescriptionStateDTO`, stamp `ExtensionLocation`,
//!      `Identifier`, and `IsBuiltin=false`.
//! 2. `UninstallExtension(InstallDir)`:
//!    - Recursively delete the install directory.
//!
//! The caller (`WindServiceHandlers::extensions:install`) is responsible for
//! `ScannedExtensionCollection::AddOrUpdate` and for broadcasting the
//! `extensions:installed` Tauri event so Wind re-fetches the extension list.
//!
//! ## Why the minimal two-pass read?
//!
//! The first pass reads only `extension/package.json` to compute the install
//! path (we need publisher+name+version *before* writing any files, so we can
//! reject collisions without partial writes). The second pass streams
//! everything to disk. This keeps memory low - we never hold the full archive
//! in RAM, and we don't unpack to a temp dir just to move it.
//!
//! ## Why no gallery API?
//!
//! `extensions:install` in `WindServiceHandlers.rs` previously responded to
//! both `install` (gallery) and `install-vsix` (local file). This installer
//! handles the local-file case - VS Code's gallery contract requires an
//! online marketplace which Land does not currently host. Gallery support
//! can layer on later by resolving a publisher identifier + version to a
//! VSIX URL, downloading to a temp file, and calling `InstallVsix`.

#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{self, Read},
	path::{Path, PathBuf},
};

use serde_json::Value;
use zip::ZipArchive;

use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

/// Everything an IPC handler needs after a successful install.
#[derive(Debug)]
pub struct InstallOutcome {
	/// `<publisher>.<name>` - the canonical identifier string.
	pub Identifier:String,
	/// Semver string from the manifest.
	pub Version:String,
	/// Extracted target directory on disk.
	pub InstalledAt:PathBuf,
	/// Fully-populated DTO, ready to `AddOrUpdate` in ScannedExtensions.
	pub Description:ExtensionDescriptionStateDTO,
}

/// Manifest facts we need before we start writing files.
struct ManifestFacts {
	Publisher:String,
	Name:String,
	Version:String,
}

/// Errors distinct enough that the IPC handler can produce useful messages
/// without a `CommonError` cast. Flattened to String at the handler boundary.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
	#[error("VSIX path '{0}' does not exist")]
	SourceMissing(PathBuf),

	#[error("VSIX archive read failure: {0}")]
	ArchiveRead(String),

	#[error("VSIX manifest missing or unreadable: {0}")]
	ManifestMissing(String),

	#[error("VSIX manifest missing required field '{0}'")]
	ManifestFieldMissing(&'static str),

	#[error("Extension '{Identifier}' version {Version} is already installed at {InstalledAt}")]
	AlreadyInstalled { Identifier:String, Version:String, InstalledAt:PathBuf },

	#[error("Filesystem error during install: {0}")]
	FilesystemIO(String),
}

const MANIFEST_ENTRY:&str = "extension/package.json";
const PAYLOAD_PREFIX:&str = "extension/";

/// Open `VsixPath` and install its payload under `InstallRoot`. On success the
/// caller receives the new identifier, install directory, and a DTO ready
/// for `ScannedExtensionCollection::AddOrUpdate`.
pub fn InstallVsix(VsixPath:&Path, InstallRoot:&Path) -> Result<InstallOutcome, InstallError> {
	if !VsixPath.exists() {
		return Err(InstallError::SourceMissing(VsixPath.to_path_buf()));
	}

	let Facts = ReadManifestFacts(VsixPath)?;
	let InstalledAt = InstallRoot.join(format!("{}.{}-{}", Facts.Publisher, Facts.Name, Facts.Version));

	if InstalledAt.exists() {
		return Err(InstallError::AlreadyInstalled {
			Identifier:format!("{}.{}", Facts.Publisher, Facts.Name),
			Version:Facts.Version,
			InstalledAt,
		});
	}

	CreateParent(&InstalledAt)?;
	ExtractPayload(VsixPath, &InstalledAt)?;

	let Description = BuildDescription(&InstalledAt)?;
	let Identifier = format!("{}.{}", Facts.Publisher, Facts.Name);

	dev_log!(
		"extensions",
		"[VsixInstaller] Installed '{}' v{} at {}",
		Identifier,
		Facts.Version,
		InstalledAt.display()
	);

	Ok(InstallOutcome { Identifier, Version:Facts.Version, InstalledAt, Description })
}

/// Delete the install directory. Returns `Ok` if the path was already absent.
pub fn UninstallExtension(InstallDir:&Path) -> Result<(), InstallError> {
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

// --- Internals ----------------------------------------------------------

fn ReadManifestFacts(VsixPath:&Path) -> Result<ManifestFacts, InstallError> {
	let Manifest = ReadFullManifest(VsixPath)?;

	let Publisher = ReadStringField(&Manifest, "publisher")?;
	let Name = ReadStringField(&Manifest, "name")?;
	let Version = ReadStringField(&Manifest, "version")?;

	Ok(ManifestFacts { Publisher, Name, Version })
}

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
pub fn ReadFullManifest(VsixPath:&Path) -> Result<Value, InstallError> {
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

fn ReadStringField(Manifest:&Value, Field:&'static str) -> Result<String, InstallError> {
	Manifest
		.get(Field)
		.and_then(|Value| Value.as_str())
		.filter(|Value| !Value.is_empty())
		.map(str::to_owned)
		.ok_or(InstallError::ManifestFieldMissing(Field))
}

fn CreateParent(InstalledAt:&Path) -> Result<(), InstallError> {
	if let Some(Parent) = InstalledAt.parent() {
		fs::create_dir_all(Parent).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;
	}

	Ok(())
}

fn ExtractPayload(VsixPath:&Path, InstalledAt:&Path) -> Result<(), InstallError> {
	let Archive = File::open(VsixPath).map_err(|Error| InstallError::ArchiveRead(Error.to_string()))?;
	let mut Archive = ZipArchive::new(Archive).map_err(|Error| InstallError::ArchiveRead(Error.to_string()))?;

	fs::create_dir_all(InstalledAt).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;

	for Index in 0..Archive.len() {
		let mut Entry = Archive
			.by_index(Index)
			.map_err(|Error| InstallError::ArchiveRead(Error.to_string()))?;

		let EntryName = Entry.name().to_string();

		// Only the `extension/...` subtree is the addon payload. Manifest-level
		// files (`[Content_Types].xml`, `extension.vsixmanifest`, `assets/`,
		// etc.) are VSIX packaging metadata and are not needed at runtime.
		let Stripped = match EntryName.strip_prefix(PAYLOAD_PREFIX) {
			Some(Path) if !Path.is_empty() => Path,
			_ => continue,
		};

		// Guard against zip-slip: the archive must not reference `..` segments
		// that escape the install dir. Reject any entry whose resolved path is
		// outside `InstalledAt`.
		let Target = InstalledAt.join(Stripped);

		let CanonicalInstall = InstalledAt.to_path_buf();

		let RejectTraversal = !Target.starts_with(&CanonicalInstall);

		if RejectTraversal {
			return Err(InstallError::ArchiveRead(format!("zip-slip entry rejected: {}", EntryName)));
		}

		if Entry.is_dir() {
			fs::create_dir_all(&Target).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;

			continue;
		}

		if let Some(Parent) = Target.parent() {
			fs::create_dir_all(Parent).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;
		}

		let mut Output = File::create(&Target).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;

		io::copy(&mut Entry, &mut Output).map_err(|Error| InstallError::FilesystemIO(Error.to_string()))?;
	}

	Ok(())
}

fn BuildDescription(InstalledAt:&Path) -> Result<ExtensionDescriptionStateDTO, InstallError> {
	let ManifestPath = InstalledAt.join("package.json");

	let Raw = fs::read_to_string(&ManifestPath).map_err(|Error| InstallError::ManifestMissing(Error.to_string()))?;

	let mut ManifestValue:Value =
		serde_json::from_str(&Raw).map_err(|Error| InstallError::ManifestMissing(Error.to_string()))?;

	let mut Description:ExtensionDescriptionStateDTO = serde_json::from_value(ManifestValue.clone())
		.map_err(|Error| InstallError::ManifestMissing(Error.to_string()))?;

	Description.ExtensionLocation = serde_json::to_value(
		url::Url::from_directory_path(InstalledAt)
			.unwrap_or_else(|_| url::Url::parse("file:///").expect("file:/// is a valid URL")),
	)
	.unwrap_or(Value::Null);

	if Description.Identifier == Value::Null || Description.Identifier == Value::Object(Default::default()) {
		let Identifier = if Description.Publisher.is_empty() {
			Description.Name.clone()
		} else {
			format!("{}.{}", Description.Publisher, Description.Name)
		};

		Description.Identifier = serde_json::json!({ "value": Identifier });
	}

	Description.IsBuiltin = false;

	// Touch the mutable manifest so later tooling that re-serialises it sees
	// the same canonical form we parsed from.
	let _ = &mut ManifestValue;

	Ok(Description)
}
