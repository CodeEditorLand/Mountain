//! `VsixInstaller::HealExecutableBits`

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

/// Walk an installed extension directory and chmod +x any file that
/// matches the same executable heuristic as fresh installs. Used on the
/// idempotent reinstall path so users who installed extensions before
/// the exec-bit promotion landed don't need to manually `chmod` shipped
/// binaries (`rust-analyzer/server/rust-analyzer`,
/// `openai.chatgpt/bin/<triple>/codex`, `Dart-Code/bin/dart`, etc.).
///
/// Errors are swallowed - this is a best-effort heal, never the reason
/// an Install fails. A file we can't open or stat just keeps its
/// existing mode and the extension's `spawn` will surface the same
/// EACCES it would have anyway.
#[cfg(unix)]
pub fn Fn(InstalledAt:&Path) {
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
			let Path = Entry.Path();

			let Ok(Metadata) = Entry.metadata() else {
				continue;
			};

			if Metadata.is_dir() {
				// Skip the bundled-deps tree by name - chmod-ing every
				// file under node_modules is wasteful and chmod-ing
				// `.bin` shims is what the npm Install lifecycle
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
