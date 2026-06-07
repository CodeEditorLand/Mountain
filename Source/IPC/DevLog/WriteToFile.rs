//! Append a single formatted line to the session's
//! `Mountain.dev.log`. The file sink is lazy: opens on first
//! call, no-ops if `Record=0` or the directory cannot be
//! created. Flushes per line so `tail -f` shows live output.

use std::{
	fs::{File, OpenOptions, create_dir_all},
	io::{BufWriter, Write as IoWrite},
	path::PathBuf,
	sync::{Mutex, OnceLock},
};

use crate::IPC::DevLog::{AppDataPrefix, IsEnabled, IsShort, SessionTimestamp};

static LOG_FILE:OnceLock<Mutex<Option<BufWriter<File>>>> = OnceLock::new();

pub fn Fn(Line:&str) {
	let Sink = InitFileSink();

	if let Ok(mut Guard) = Sink.lock() {
		if let Some(Writer) = Guard.as_mut() {
			let _ = Writer.write_all(Line.as_bytes());

			if !Line.ends_with('\n') {
				let _ = Writer.write_all(b"\n");
			}

			let _ = Writer.flush();
		}
	}
}

pub(super) fn InitFileSink() -> &'static Mutex<Option<BufWriter<File>>> {
	LOG_FILE.get_or_init(|| {
		if !FileSinkEnabled() {
			return Mutex::new(None);
		}

		let Dir = ResolveLogDirectory();

		if create_dir_all(&Dir).is_err() {
			eprintln!("[DEV:LOG] Failed to create log directory {}", Dir.display());

			return Mutex::new(None);
		}

		let Path = Dir.join("Mountain.dev.log");

		match OpenOptions::new().create(true).append(true).open(&Path) {
			Ok(File) => {
				let mut Writer = BufWriter::with_capacity(64 * 1024, File);

				let Header = format!(
					"# Land dev log - started {}, pid {}, short={}, ipc-enabled={}\n",
					SessionTimestamp::Fn(),
					std::process::id(),
					IsShort::Fn(),
					IsEnabled::Fn("ipc"),
				);

				let _ = Writer.write_all(Header.as_bytes());

				let _ = Writer.flush();

				eprintln!("[DEV:LOG] File sink → {}", Path.display());

				Mutex::new(Some(Writer))
			},
			Err(Error) => {
				eprintln!("[DEV:LOG] Failed to open {}: {}", Path.display(), Error);

				Mutex::new(None)
			},
		}
	})
}

fn FileSinkEnabled() -> bool {
	static ENABLED:OnceLock<bool> = OnceLock::new();

	*ENABLED.get_or_init(|| {
		match std::env::var("Record") {
			Ok(Value) => matches!(Value.as_str(), "1" | "true" | "yes" | "on"),
			Err(_) => cfg!(debug_assertions) && std::env::var("Trace").is_ok(),
		}
	})
}

fn ResolveLogDirectory() -> PathBuf {
	let Stamp = SessionTimestamp::Fn();

	let Base = match AppDataPrefix::Fn() {
		Some(Prefix) => PathBuf::from(Prefix).join("logs"),

		None => std::env::temp_dir().join("land-editor-logs"),
	};

	Base.join(Stamp)
}
