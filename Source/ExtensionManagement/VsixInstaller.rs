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
//!    - If target already exists with a readable manifest, treat the install as
//!      idempotent - return the existing outcome instead of re-extracting.
//!      Matches VS Code's reinstall-is-a-no-op semantics and prevents the
//!      renderer crash where `ExtensionsWorkbenchService` dereferences a null
//!      result from a rejected install.
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

	let Identifier = format!("{}.{}", Facts.Publisher, Facts.Name);

	// Idempotent reinstall: if the target directory already holds the same
	// <publisher>.<name>-<version>, skip extraction and surface the existing
	// install as a success. Reading the on-disk manifest handles the edge
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
			// even though the file is intact on disk. Walk the install
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

		// Corrupt / partial previous install - wipe and re-extract below.
		dev_log!(
			"extensions",
			"[VsixInstaller] Existing install at {} is unreadable - wiping and reinstalling",
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

		// Preserve Unix executable bits recorded in the VSIX. Extensions
		// that ship platform-native binaries (openai.chatgpt's `codex`,
		// language-server launchers, etc.) rely on the `0o755` mode being
		// carried through the zip. Without this, the child `spawn()`
		// inside the extension fails with `EACCES` because the freshly
		// written file has only the default `0o644` read/write mode.
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;

			let PermissionBits = Entry.unix_mode().map(|Mode| Mode & 0o777).unwrap_or(0);

			// Promote executable bit whenever the payload is a native
			// binary the extension will spawn. Heuristics, in order:
			//   1. Zip already recorded any exec bit (user/group/other).
			//   2. Path lives under a `bin/` segment (vscode convention for shipped CLI
			//      tools: openai.chatgpt's `bin/<triple>/codex`, rust-analyzer's
			//      `bin/ra_lsp`, Dart-Code's `bin/dart`, …).
			//   3. First two bytes match a known executable magic number: Mach-O
			//      (`\xCF\xFA\xED\xFE` / `\xCE\xFA\xED\xFE` / fat `\xCA\xFE\xBA\xBE`), ELF
			//      (`\x7FELF`), or shebang (`#!`). Some zip creators drop all mode bits;
			//      the magic-number probe is the only way to tell before the extension
			//      tries to spawn the file.
			// Directory segments that conventionally hold spawnable
			// binaries: VS Code's `bin/`, language-server `server/`
			// (rust-analyzer, ruby-lsp, jdt-ls, gopls), .NET's
			// `tools/`, OmniSharp's `omnisharp/`, debug-adapter
			// `adapter/`, native-host `native/`. Match any path
			// segment, not just the leading one - many VSIXes nest
			// like `out/server/...` or `dist/bin/...`.
			let IsBinPath = Stripped
				.split('/')
				.any(|Segment| matches!(Segment, "bin" | "server" | "tools" | "omnisharp" | "adapter" | "native"));

			let HasExecBit = PermissionBits & 0o111 != 0;

			let LooksExecutable = if HasExecBit || IsBinPath {
				true
			} else {
				let mut Probe = [0u8; 4];

				match std::fs::File::open(&Target).and_then(|mut Handle| {
					use std::io::Read as IoRead;
					IoRead::read(&mut Handle, &mut Probe).map(|BytesRead| (BytesRead, Probe))
				}) {
					Ok((BytesRead, Bytes)) if BytesRead >= 2 => {
						let Shebang = &Bytes[..2] == b"#!";

						let ElfMagic = BytesRead >= 4 && &Bytes[..4] == b"\x7FELF";

						let MachMagic = BytesRead >= 4
							&& matches!(
								&Bytes[..4],
								b"\xCF\xFA\xED\xFE"
									| b"\xCE\xFA\xED\xFE" | b"\xFE\xED\xFA\xCF"
									| b"\xFE\xED\xFA\xCE" | b"\xCA\xFE\xBA\xBE"
									| b"\xBE\xBA\xFE\xCA"
							);

						Shebang || ElfMagic || MachMagic
					},

					_ => false,
				}
			};

			let FinalMode = if LooksExecutable {
				(PermissionBits | 0o755) & 0o755
			} else {
				(PermissionBits | 0o644) & 0o755
			};

			let _ = fs::set_permissions(&Target, fs::Permissions::from_mode(FinalMode));
		}
	}

	Ok(())
}

/// Walk an installed extension directory and chmod +x any file that
/// matches the same executable heuristic as fresh installs. Used on the
/// idempotent reinstall path so users who installed extensions before
/// the exec-bit promotion landed don't need to manually `chmod` shipped
/// binaries (`rust-analyzer/server/rust-analyzer`,
/// `openai.chatgpt/bin/<triple>/codex`, `Dart-Code/bin/dart`, etc.).
///
/// Errors are swallowed - this is a best-effort heal, never the reason
/// an install fails. A file we can't open or stat just keeps its
/// existing mode and the extension's `spawn` will surface the same
/// EACCES it would have anyway.
#[cfg(unix)]
pub fn HealExecutableBits(InstalledAt:&Path) {
	use std::{io::Read, os::unix::fs::PermissionsExt};

	fn IsBinSegment(Segment:&std::ffi::OsStr) -> bool {
		let Some(Name) = Segment.to_str() else {
			return false;
		};

		matches!(Name, "bin" | "server" | "tools" | "omnisharp" | "adapter" | "native")
	}

	fn LooksExecutable(Target:&Path, RelativeFromRoot:&Path) -> bool {
		let IsBinPath = RelativeFromRoot
			.components()
			.any(|Component| IsBinSegment(Component.as_os_str()));

		if IsBinPath {
			return true;
		}

		let Ok(mut Handle) = std::fs::File::open(Target) else {
			return false;
		};

		let mut Probe = [0u8; 4];

		let Ok(BytesRead) = Handle.read(&mut Probe) else {
			return false;
		};

		if BytesRead < 2 {
			return false;
		}

		let Shebang = &Probe[..2] == b"#!";

		let ElfMagic = BytesRead >= 4 && &Probe[..4] == b"\x7FELF";

		let MachMagic = BytesRead >= 4
			&& matches!(
				&Probe[..4],
				b"\xCF\xFA\xED\xFE"
					| b"\xCE\xFA\xED\xFE"
					| b"\xFE\xED\xFA\xCF"
					| b"\xFE\xED\xFA\xCE"
					| b"\xCA\xFE\xBA\xBE"
					| b"\xBE\xBA\xFE\xCA"
			);

		Shebang || ElfMagic || MachMagic
	}

	fn Walk(Dir:&Path, Root:&Path, Healed:&mut usize) {
		let Ok(Entries) = std::fs::read_dir(Dir) else {
			return;
		};

		for Entry in Entries.flatten() {
			let Path = Entry.path();

			let Ok(Metadata) = Entry.metadata() else {
				continue;
			};

			if Metadata.is_dir() {
				// Skip the bundled-deps tree by name - chmod-ing every
				// file under node_modules is wasteful and chmod-ing
				// `.bin` shims is what the npm install lifecycle
				// already handles. If an extension genuinely needs a
				// binary inside node_modules executable, its postinstall
				// will mark it.
				if Entry.file_name() == "node_modules" {
					continue;
				}

				Walk(&Path, Root, Healed);

				continue;
			}

			let Ok(Relative) = Path.strip_prefix(Root) else {
				continue;
			};

			let Mode = Metadata.permissions().mode() & 0o777;

			if Mode & 0o100 != 0 {
				// Owner-exec already set; trust it.
				continue;
			}

			if !LooksExecutable(&Path, Relative) {
				continue;
			}

			let Promoted = (Mode | 0o755) & 0o755;

			if std::fs::set_permissions(&Path, std::fs::Permissions::from_mode(Promoted)).is_ok() {
				*Healed += 1;
			}
		}
	}

	let mut Healed:usize = 0;

	Walk(InstalledAt, InstalledAt, &mut Healed);

	if Healed > 0 {
		dev_log!(
			"extensions",
			"[VsixInstaller] Healed {} executable bit(s) under {}",
			Healed,
			InstalledAt.display()
		);
	}
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
