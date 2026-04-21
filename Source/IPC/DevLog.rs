//! # DevLog - Tag-filtered development logging
//!
//! Controlled by `LAND_DEV_LOG` environment variable.
//! The same tags work in both Mountain (Rust) and Wind/Sky (TypeScript).
//!
//! ## Usage
//! ```bash
//! LAND_DEV_LOG=vfs,ipc ./Mountain          # only VFS + IPC
//! LAND_DEV_LOG=all ./Mountain              # everything
//! LAND_DEV_LOG=short ./Mountain            # everything, compressed + deduped
//! LAND_DEV_LOG=terminal,exthost ./Mountain # terminal + extension host
//! ./Mountain                               # nothing (only normal log!() output)
//! ```
//!
//! ## Short Mode
//!
//! `LAND_DEV_LOG=short` enables all tags but compresses output:
//! - Long app-data paths aliased to `$APP`
//! - Consecutive duplicate messages counted (`(x14)` suffix)
//! - Rust log targets compressed (`D::Binary::Main::Entry` → `Entry`)
//!
//! ## File sink
//!
//! When `LAND_DEV_LOG_FILE=1` (or, in debug builds, `LAND_DEV_LOG` is set),
//! every emitted line is mirrored to:
//!
//! ```
//! ~/Library/Application Support/<bundle>/logs/<YYYYMMDDTHHMMSS>/Mountain.dev.log
//! ```
//!
//! The timestamp directory follows Tauri's `tauri-plugin-log` format, so
//! the dev log sits next to the plugin's own log file for the same boot
//! (one directory per process start). Use `LAND_DEV_LOG_FILE=0` to force-
//! disable even in debug. File writes are flushed per line so `tail -f`
//! shows live output.
//!
//! ## Tags (38 granular tags across all Elements)
//!
//! | Tag           | Scope                                               |
//! |---------------|-----------------------------------------------------|
//! | `vfs`         | File stat, read, write, readdir, mkdir, delete, copy|
//! | `ipc`         | IPC routing: invoke dispatch, channel calls          |
//! | `config`      | Configuration get/set, env paths, workbench config   |
//! | `lifecycle`   | Startup, shutdown, phases, window events             |
//! | `storage`     | Storage get/set/delete, items, optimize              |
//! | `folder`      | Folder picker, workspace navigation                  |
//! | `exthost`     | Extension host: create, start, kill, exit info       |
//! | `extensions`  | Extension scanning, activation, management           |
//! | `terminal`    | Terminal/PTY: create, sendText, profiles, shell      |
//! | `search`      | Search: findFiles, findInFiles                       |
//! | `themes`      | Theme: list, get active, set                         |
//! | `window`      | Window: focus, maximize, minimize, fullscreen        |
//! | `nativehost`  | OS integration: process, devtools, shell             |
//! | `clipboard`   | Clipboard: read/write text, buffer, image            |
//! | `commands`    | Command registry: execute, getAll                    |
//! | `model`       | Text model: open, close, get, updateContent          |
//! | `output`      | Output channels: create, append, show                |
//! | `notification`| Notifications: show, progress                        |
//! | `progress`    | Progress: begin, end, report                         |
//! | `quickinput`  | Quick input: showQuickPick, showInputBox             |
//! | `workingcopy` | Working copy: dirty state                            |
//! | `workspaces`  | Workspace: folders, recent, enter                    |
//! | `keybinding`  | Keybindings: add, remove, lookup                     |
//! | `label`       | Label service: getBase, getUri                       |
//! | `history`     | Navigation history: push, goBack, goForward          |
//! | `decorations` | Decorations: get, set, clear                         |
//! | `textfile`    | Text file operations: read, write, save              |
//! | `update`      | Update service: check, download, apply               |
//! | `encryption`  | Encryption: encrypt, decrypt                         |
//! | `menubar`     | Menubar updates                                      |
//! | `url`         | URL handler: registerExternalUriOpener               |
//! | `grpc`        | gRPC/Vine: server, client, connections               |
//! | `cocoon`      | Cocoon sidecar: spawn, health, handshake             |
//! | `bootstrap`   | Effect-TS bootstrap stages                           |
//! | `preload`     | Preload: globals, polyfills, ipcRenderer             |

use std::{
	fs::{File, OpenOptions, create_dir_all},
	io::{BufWriter, Write as IoWrite},
	path::PathBuf,
	sync::{
		Mutex,
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
	time::{SystemTime, UNIX_EPOCH},
};

static ENABLED_TAGS:OnceLock<Vec<String>> = OnceLock::new();
static SHORT_MODE:OnceLock<bool> = OnceLock::new();

// ── File sink ────────────────────────────────────────────────────────────
//
// Mirrors every `dev_log!` line to a timestamped file under the app's
// `logs/<YYYYMMDDTHHMMSS>/` directory so long sessions can be inspected
// post-mortem without scrolling terminal history. Enable with:
//
//     LAND_DEV_LOG_FILE=1     # explicit opt-in
//     LAND_DEV_LOG_FILE=0     # explicit opt-out (wins over defaults)
//
// When unset, file logging is auto-enabled in debug builds iff
// `LAND_DEV_LOG` itself is set to at least one tag. Release builds stay
// silent unless the opt-in flag is present, to avoid surprise writes in
// shipped binaries.
//
// Directory layout matches Tauri's tauri-plugin-log output (same parent
// `logs/` and same `YYYYMMDDTHHMMSS` subdir format) so the two logs sit
// side by side when the user greps one timestamp.

static LOG_FILE:OnceLock<Mutex<Option<BufWriter<File>>>> = OnceLock::new();

/// Decide whether the file sink should be active. Returns the final flag
/// once per process; subsequent calls are cached.
fn FileSinkEnabled() -> bool {
	static ENABLED:OnceLock<bool> = OnceLock::new();
	*ENABLED.get_or_init(|| {
		match std::env::var("LAND_DEV_LOG_FILE") {
			Ok(Value) => matches!(Value.as_str(), "1" | "true" | "yes" | "on"),
			Err(_) => {
				// Auto-enable when LAND_DEV_LOG is set in a debug build.
				cfg!(debug_assertions) && std::env::var("LAND_DEV_LOG").is_ok()
			},
		}
	})
}

/// Resolve the log directory for this session:
///   ~/Library/Application Support/<bundle>/logs/<YYYYMMDDTHHMMSS>/
///
/// The timestamp is picked once per process at first call. If the app-data
/// prefix can't be detected (Tauri hasn't spawned yet, say), fall back to
/// the system temp directory with the same naming — dev_log still works
/// and the file ends up somewhere predictable.
fn ResolveLogDirectory() -> PathBuf {
	let Stamp = FormatTimestamp();
	let Base = match AppDataPrefix() {
		Some(Prefix) => PathBuf::from(Prefix).join("logs"),
		None => std::env::temp_dir().join("land-editor-logs"),
	};
	Base.join(Stamp)
}

/// Session timestamp in local time, cached once per process. MUST match
/// whatever `WindServiceHandlers.rs::"nativeHost:getEnvironmentPaths"`
/// builds, because VS Code's file service writes `window1/output/*.log`
/// into the directory that handler returns — if DevLog and VS Code use
/// different timezones, `Mountain.dev.log` and the `window1/` subtree
/// land in two sibling directories 2–3 hours apart, which makes every
/// post-mortem investigation start with "which folder has the real
/// log?". Picking `chrono::Local::now()` matches the VS Code convention
/// (Tauri's tauri-plugin-log also writes local-time `YYYYMMDDTHHMMSS`).
///
/// The format string is deliberately identical to the handler's
/// `"%Y%m%dT%H%M%S"`, and both sides pull from the same OnceLock via
/// `SessionTimestamp()` so re-entrant calls from anywhere in the
/// codebase produce the same string.
pub fn SessionTimestamp() -> String {
	static STAMP:OnceLock<String> = OnceLock::new();
	STAMP
		.get_or_init(|| chrono::Local::now().format("%Y%m%dT%H%M%S").to_string())
		.clone()
}

fn FormatTimestamp() -> String { SessionTimestamp() }

// `DaysToYMD` + `IsLeap` were previously used to build a UTC timestamp
// string without pulling chrono into DevLog. Replaced by
// `chrono::Local::now()` in `SessionTimestamp()` so this file agrees
// with `WindServiceHandlers.rs::"nativeHost:getEnvironmentPaths"` on
// the session-log directory. chrono is already a Mountain dependency,
// so the vendored date math is dead weight now.

/// Initialise the file sink on first call. Silently falls through to a
/// disabled sink if the directory or file can't be created — the caller
/// must never panic because of a log-file failure.
fn InitFileSink() -> &'static Mutex<Option<BufWriter<File>>> {
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
				// Header pins the boot-time context so every session's file
				// is self-describing even without the surrounding terminal.
				let Header = format!(
					"# Land dev log — started {}, pid {}, tags={:?}, short={}\n",
					FormatTimestamp(),
					std::process::id(),
					EnabledTags(),
					IsShort(),
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

/// Append a single formatted line to the session's log file if the file
/// sink is active. Swallows every error — dev_log must never crash.
pub fn WriteToFile(Line:&str) {
	let Sink = InitFileSink();
	if let Ok(mut Guard) = Sink.lock() {
		if let Some(Writer) = Guard.as_mut() {
			let _ = Writer.write_all(Line.as_bytes());
			if !Line.ends_with('\n') {
				let _ = Writer.write_all(b"\n");
			}
			// Flush on every line — ordering/tail-f matters more than throughput
			// for dev logs, and the BufWriter coalesces partial writes anyway.
			let _ = Writer.flush();
		}
	}
}

// ── Path alias ──────────────────────────────────────────────────────────
// The app-data directory name is absurdly long. In short mode, alias it.
static APP_DATA_PREFIX:OnceLock<Option<String>> = OnceLock::new();

/// Produce an identity signature for THIS running binary derived from
/// `CARGO_PKG_NAME` (which Maintain sets to the long PascalCase product
/// name before `cargo build`). Each profile produces a distinct signature
/// — `_Debug_Mountain` → `.debug.mountain`, `_Compile_Mountain` →
/// `.compile.mountain`, `_Bundle_Clean_Debug_ElectronProfile_Mountain`
/// → `.debug.electron.profile.mountain` — so a candidate directory in
/// `~/Library/Application Support/` can be disambiguated against every
/// other `land.editor.*.mountain` leftover from prior runs.
///
/// Only the last three underscore-delimited segments are used: the
/// leading `DevelopmentNodeEnvironment_MicrosoftVSCodeDependency_
/// 22NodeVersion_Bundle_Clean` prefix is identical across profiles and
/// doesn't help disambiguate, while the tail (`Debug_Mountain` vs
/// `Compile_Mountain` vs `Debug_ElectronProfile_Mountain`) is where the
/// per-profile identity lives.
fn BinarySignature() -> String {
	let PackageName = env!("CARGO_PKG_NAME");
	let Segments:Vec<&str> = PackageName.split('_').collect();
	let Take = Segments.len().min(4);
	let Start = Segments.len().saturating_sub(Take);
	let Tail = Segments[Start..].join("_");
	// Lowercase + `_` → `.` gives the same segment order the Tauri
	// identifier uses (identifiers are dot-delimited lowercase). We
	// don't split PascalCase into words here — the substring match
	// below doesn't need exact equality, just a unique tail.
	Tail.to_ascii_lowercase().replace('_', ".")
}

fn DetectAppDataPrefix() -> Option<String> {
	let Home = std::env::var("HOME").ok()?;
	let Base = format!("{}/Library/Application Support", Home);
	let Signature = BinarySignature();

	// Prefer a directory whose name ends with this binary's unique tail
	// signature: that's the app-data directory Tauri created for THIS
	// profile. Without this check, a user who has ever launched another
	// profile (debug-electron, release-electron, …) will see DevLog
	// writing into that stale directory while userdata still goes into
	// the current one, producing an "empty logs folder" mystery.
	let mut FirstMatchingMountain:Option<String> = None;
	if let Ok(Entries) = std::fs::read_dir(&Base) {
		for Entry in Entries.flatten() {
			let Name = Entry.file_name();
			let Name = Name.to_string_lossy().into_owned();
			if !Name.starts_with("land.editor.") || !Name.contains("mountain") {
				continue;
			}
			// Strict match: binary signature tail is a suffix of the dir name.
			if Name.ends_with(&Signature) {
				return Some(format!("{}/{}", Base, Name));
			}
			// Lossy match: some segment of the binary signature is contained
			// in the dir name. Used only if no strict match exists.
			if FirstMatchingMountain.is_none() {
				FirstMatchingMountain = Some(format!("{}/{}", Base, Name));
			}
		}
	}
	FirstMatchingMountain
}

/// Get the app-data path prefix for aliasing (cached).
pub fn AppDataPrefix() -> &'static Option<String> { APP_DATA_PREFIX.get_or_init(DetectAppDataPrefix) }

/// Replace the long app-data path with `$APP` in a string.
pub fn AliasPath(Input:&str) -> String {
	if let Some(Prefix) = AppDataPrefix() {
		Input.replace(Prefix.as_str(), "$APP")
	} else {
		Input.to_string()
	}
}

// ── Dedup buffer ────────────────────────────────────────────────────────

pub struct DedupState {
	pub LastKey:String,
	pub Count:u64,
}

pub static DEDUP:Mutex<DedupState> = Mutex::new(DedupState { LastKey:String::new(), Count:0 });

/// Flush the dedup buffer - prints the pending count if > 1.
pub fn FlushDedup() {
	if let Ok(mut State) = DEDUP.lock() {
		if State.Count > 1 {
			let Tail = format!("  (x{})", State.Count);
			eprintln!("{}", Tail);
			WriteToFile(&Tail);
		}
		State.LastKey.clear();
		State.Count = 0;
	}
}

// ── Benign-probe classification (BATCH-17) ───────────────────────────────
//
// Extensions and the workbench probe dozens of optional files on every boot
// (VS Code + Copilot + language-features). `stat ENOENT` lines for these
// paths are functionally noise: they confirm the probe exists but nothing
// acts on a failure. Three steps keep the log useful:
//
//   1. Known-optional patterns downgrade to Debug level (suppressed from the
//      default dev-log stream, still written to the file sink).
//   2. Per-unique-path dedup: log the first miss once per session via
//      [`DebugOnce`]; later hits on the same path are swallowed.
//   3. Virtual resource 404s (`vscode://`, cached globalStorage paths) are
//      matched by the same helper so `BATCH-06`'s earlier suppression stays in
//      one place.

const BENIGN_ENOENT_SUBSTRINGS:&[&str] = &[
	// VS Code / Claude / Copilot probe paths.
	".claude/agents",
	".claude/settings.json",
	".claude/settings.local.json",
	".github/copilot",
	".github/agents",
	".vscode/settings.json",
	".vscode/launch.json",
	".vscode/extensions.json",
	"agentPlugins",
	"agent-plugins",
	"chatEditingSessions",
	"chatSessions",
	// Per-extension state probes.
	"machineid",
	"terminalSuggestGlobalsCacheV2.json",
	"globalStorage",
	// Virtual scheme misses already covered by earlier batches.
	"vscode://schemas-associations/",
];

/// Return true when the given path is a known-optional probe whose absence
/// is never an error condition. Used to downgrade `stat ENOENT` spam.
pub fn IsBenignEnoent(Path:&str) -> bool { BENIGN_ENOENT_SUBSTRINGS.iter().any(|Needle| Path.contains(Needle)) }

static DEBUG_ONCE_KEYS:OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();

fn DebugOnceKeys() -> &'static Mutex<std::collections::HashSet<String>> {
	DEBUG_ONCE_KEYS.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

/// Emit the line exactly once per process, keyed on the supplied key. Later
/// calls with the same key are silently dropped. The file sink still
/// captures the very first occurrence so the probe is documented.
pub fn DebugOnce(Tag:&str, Key:&str, Line:&str) {
	if let Ok(mut Keys) = DebugOnceKeys().lock() {
		if !Keys.insert(Key.to_string()) {
			return;
		}
	}
	if IsEnabled(Tag) || IsEnabled("all") {
		let Formatted = format!("[DEV:{}] {}", Tag.to_uppercase(), Line);
		eprintln!("{}", Formatted);
		WriteToFile(&Formatted);
	} else {
		// Never echo to the console, but preserve in the file sink so
		// post-mortems still see which probe paths fired.
		let Formatted = format!("[DEV:{}/once] {}", Tag.to_uppercase(), Line);
		WriteToFile(&Formatted);
	}
}

// ── Tag resolution ──────────────────────────────────────────────────────

fn EnabledTags() -> &'static Vec<String> {
	ENABLED_TAGS.get_or_init(|| {
		match std::env::var("LAND_DEV_LOG") {
			Ok(Val) => Val.split(',').map(|S| S.trim().to_lowercase()).collect(),
			Err(_) => vec![],
		}
	})
}

/// Whether `LAND_DEV_LOG=short` is active.
pub fn IsShort() -> bool { *SHORT_MODE.get_or_init(|| EnabledTags().iter().any(|T| T == "short")) }

/// Check if a tag is enabled.
pub fn IsEnabled(Tag:&str) -> bool {
	let Tags = EnabledTags();
	if Tags.is_empty() {
		return false;
	}
	let Lower = Tag.to_lowercase();
	Tags.iter().any(|T| T == "all" || T == "short" || T == Lower.as_str())
}

/// Log a tagged dev message. Only prints if the tag is enabled via
/// LAND_DEV_LOG.
///
/// In `short` mode: aliases long paths, deduplicates consecutive identical
/// lines.
#[macro_export]
macro_rules! dev_log {
	($Tag:expr, $($Arg:tt)*) => {
		if $crate::IPC::DevLog::IsEnabled($Tag) {
			let RawMessage = format!($($Arg)*);
			let TagUpper = $Tag.to_uppercase();
			if $crate::IPC::DevLog::IsShort() {
				let Aliased = $crate::IPC::DevLog::AliasPath(&RawMessage);
				let Key = format!("{}:{}", TagUpper, Aliased);
				let ShouldPrint = {
					if let Ok(mut State) = $crate::IPC::DevLog::DEDUP.lock() {
						if State.LastKey == Key {
							State.Count += 1;
							false
						} else {
							let PrevCount = State.Count;
							let HadPrev = !State.LastKey.is_empty();
							State.LastKey = Key;
							State.Count = 1;
							if HadPrev && PrevCount > 1 {
								let Tail = format!("  (x{})", PrevCount);
								eprintln!("{}", Tail);
								$crate::IPC::DevLog::WriteToFile(&Tail);
							}
							true
						}
					} else {
						true
					}
				};
				if ShouldPrint {
					let Formatted = format!("[DEV:{}] {}", TagUpper, Aliased);
					eprintln!("{}", Formatted);
					$crate::IPC::DevLog::WriteToFile(&Formatted);
				}
			} else {
				let Formatted = format!("[DEV:{}] {}", TagUpper, RawMessage);
				eprintln!("{}", Formatted);
				$crate::IPC::DevLog::WriteToFile(&Formatted);
			}
		}
	};
}

// ============================================================================
// OTLP Span Emission — sends spans directly to Jaeger/OTEL collector
// ============================================================================

static OTLP_AVAILABLE:AtomicBool = AtomicBool::new(true);
static OTLP_TRACE_ID:OnceLock<String> = OnceLock::new();

fn GetTraceId() -> &'static str {
	OTLP_TRACE_ID.get_or_init(|| {
		use std::{
			collections::hash_map::DefaultHasher,
			hash::{Hash, Hasher},
		};
		let mut H = DefaultHasher::new();
		std::process::id().hash(&mut H);
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_nanos()
			.hash(&mut H);
		format!("{:032x}", H.finish() as u128)
	})
}

pub fn NowNano() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64 }

/// Emit an OTLP span to the local collector (Jaeger at 127.0.0.1:4318).
/// Fire-and-forget on a background thread. Stops trying after first failure.
pub fn EmitOTLPSpan(Name:&str, StartNano:u64, EndNano:u64, Attributes:&[(&str, &str)]) {
	if !cfg!(debug_assertions) {
		return;
	}
	if !OTLP_AVAILABLE.load(Ordering::Relaxed) {
		return;
	}

	let SpanId = format!("{:016x}", rand_u64());
	let TraceId = GetTraceId().to_string();
	let SpanName = Name.to_string();

	let AttributesJson:Vec<String> = Attributes
		.iter()
		.map(|(K, V)| {
			format!(
				r#"{{"key":"{}","value":{{"stringValue":"{}"}}}}"#,
				K,
				V.replace('\\', "\\\\").replace('"', "\\\"")
			)
		})
		.collect();

	let IsError = SpanName.contains("error");

	let StatusCode = if IsError { 2 } else { 1 };
	let Payload = format!(
		concat!(
			r#"{{"resourceSpans":[{{"resource":{{"attributes":["#,
			r#"{{"key":"service.name","value":{{"stringValue":"land-editor-mountain"}}}},"#,
			r#"{{"key":"service.version","value":{{"stringValue":"0.0.1"}}}}"#,
			r#"]}},"scopeSpans":[{{"scope":{{"name":"mountain.ipc","version":"1.0.0"}},"#,
			r#""spans":[{{"traceId":"{}","spanId":"{}","name":"{}","kind":1,"#,
			r#""startTimeUnixNano":"{}","endTimeUnixNano":"{}","#,
			r#""attributes":[{}],"status":{{"code":{}}}}}]}}]}}]}}"#,
		),
		TraceId,
		SpanId,
		SpanName,
		StartNano,
		EndNano,
		AttributesJson.join(","),
		StatusCode,
	);

	// Fire-and-forget on a background thread
	std::thread::spawn(move || {
		use std::{
			io::{Read as IoRead, Write as IoWrite},
			net::TcpStream,
			time::Duration,
		};

		let Ok(mut Stream) = TcpStream::connect_timeout(&"127.0.0.1:4318".parse().unwrap(), Duration::from_millis(200))
		else {
			OTLP_AVAILABLE.store(false, Ordering::Relaxed);
			return;
		};
		let _ = Stream.set_write_timeout(Some(Duration::from_millis(200)));
		let _ = Stream.set_read_timeout(Some(Duration::from_millis(200)));

		let HttpReq = format!(
			"POST /v1/traces HTTP/1.1\r\nHost: 127.0.0.1:4318\r\nContent-Type: application/json\r\nContent-Length: \
			 {}\r\nConnection: close\r\n\r\n",
			Payload.len()
		);
		if Stream.write_all(HttpReq.as_bytes()).is_err() {
			return;
		}
		if Stream.write_all(Payload.as_bytes()).is_err() {
			return;
		}
		let mut Buf = [0u8; 32];
		let _ = Stream.read(&mut Buf);
		if !(Buf.starts_with(b"HTTP/1.1 2") || Buf.starts_with(b"HTTP/1.0 2")) {
			OTLP_AVAILABLE.store(false, Ordering::Relaxed);
		}
	});
}

fn rand_u64() -> u64 {
	use std::{
		collections::hash_map::DefaultHasher,
		hash::{Hash, Hasher},
	};
	let mut H = DefaultHasher::new();
	std::thread::current().id().hash(&mut H);
	NowNano().hash(&mut H);
	H.finish()
}

/// Convenience macro: emit an OTLP span for an IPC handler.
/// Usage: `otel_span!("file:readFile", StartNano, &[("path", &SomePath)]);`
#[macro_export]
macro_rules! otel_span {
	($Name:expr, $Start:expr, $Attrs:expr) => {
		$crate::IPC::DevLog::EmitOTLPSpan($Name, $Start, $crate::IPC::DevLog::NowNano(), $Attrs)
	};
	($Name:expr, $Start:expr) => {
		$crate::IPC::DevLog::EmitOTLPSpan($Name, $Start, $crate::IPC::DevLog::NowNano(), &[])
	};
}
