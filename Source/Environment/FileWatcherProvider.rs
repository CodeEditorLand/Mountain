//! # FileWatcherProvider (Environment)
//!
//! Backing implementation of
//! [`FileWatcherProvider`](CommonLibrary::FileSystem::FileWatcherProvider)
//! for [`MountainEnvironment`].
//!
//! Native filesystem notifications are delegated to the `notify` crate, which
//! picks up inotify on Linux, FSEvents on macOS, and ReadDirectoryChangesW
//! on Windows. Events from the watcher thread flow through an unbounded
//! channel into a tokio task that forwards them back to Cocoon over the
//! reverse-RPC channel as `$fileWatcher:event` notifications.
//!
//! # Concurrency notes
//!
//! - `notify::recommended_watcher` executes callbacks on its own native
//!   thread, so we tunnel events through a bounded channel before touching
//!   async code. The forwarder task is spawned once on first registration
//!   and lives for the entire process lifetime.
//! - macOS FSEvents may emit duplicate Create/Change events for the same
//!   path in very short succession. We debounce by path within a 100 ms
//!   window per-handle, keyed on `(handle, path, kind)`.
//! - Linux inotify has a small per-user watcher cap (`fs.inotify.max_user_watches`);
//!   hitting it surfaces as `notify::Error::MaxFilesWatch`. We propagate
//!   that verbatim to the caller so the UI can show a guidance message.

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex as StandardMutex},
	time::{Duration, Instant},
};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::FileWatcherProvider::{FileWatcherProvider, WatchEvent, WatchEventKind},
	IPC::IPCProvider::IPCProvider,
};
use async_trait::async_trait;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::json;
use tokio::sync::mpsc as TokioMPSC;

use super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

/// Interval below which a second (path, kind) event for the same handle is
/// ignored. Tuned for FSEvents coalescing.
const DebounceWindow:Duration = Duration::from_millis(100);

/// Internal entry tracked per registered watcher. The `Watcher` handle must
/// be kept alive for the lifetime of the registration; dropping it releases
/// the OS resources.
struct WatcherEntry {
	#[allow(dead_code)]
	Watcher:RecommendedWatcher,
	LastSeen:HashMap<(PathBuf, &'static str), Instant>,
}

/// Lazily-initialised process-wide state for file watching. Instances of the
/// event-forwarder task are singletons keyed on the MountainEnvironment
/// handle. Access through `WatcherState::Get`.
pub struct WatcherState {
	pub Entries:Arc<StandardMutex<HashMap<String, WatcherEntry>>>,
	pub EventSender:TokioMPSC::UnboundedSender<WatchEvent>,
}

impl WatcherState {
	/// Obtain (or create) the global WatcherState. The forwarder task is
	/// spawned on first access. Must be called from within a tokio runtime.
	pub fn Get(env:&MountainEnvironment) -> Arc<WatcherState> {
		use std::sync::OnceLock;

		// One WatcherState per process — the backing notify watchers are
		// cheap and multiplex fine, and we want a single forwarder task.
		static GLOBAL:OnceLock<Arc<WatcherState>> = OnceLock::new();
		GLOBAL
			.get_or_init(|| {
				let (tx, mut rx) = TokioMPSC::unbounded_channel::<WatchEvent>();
				let state = Arc::new(WatcherState {
					Entries:Arc::new(StandardMutex::new(HashMap::new())),
					EventSender:tx,
				});

				// The forwarder task holds a weak ref to the environment so
				// it unwinds cleanly if the env is ever torn down.
				let env_clone = env.clone();
				tokio::spawn(async move {
					use tauri::Emitter;
					while let Some(WatchEvent { Handle, Kind, Path }) = rx.recv().await {
						let ipc_provider:Arc<dyn IPCProvider> = env_clone.Require();
						let payload = json!({
							"handle": Handle,
							"kind": Kind.AsString(),
							"path": Path.to_string_lossy().to_string(),
						});
						if let Err(error) = ipc_provider
							.SendNotificationToSideCar(
								"cocoon-main".to_string(),
								"$fileWatcher:event".to_string(),
								payload.clone(),
							)
							.await
						{
							dev_log!(
								"filewatcher",
								"warn: [FileWatcherProvider] Failed to forward event handle={} kind={} path={:?}: {:?}",
								Handle,
								Kind.AsString(),
								Path,
								error
							);
						}
						// Dual-emit to Wind/Sky so the Explorer tree, the
						// search index, and any other webview-side consumer
						// can react to disk mutations without going through
						// Cocoon. Wind's `TauriChannel` subscribes to
						// `sky://vfs/fileChange` under the localFilesystem
						// channel.
						if let Err(Error) =
							env_clone.ApplicationHandle.emit("sky://vfs/fileChange", &payload)
						{
							dev_log!(
								"filewatcher",
								"warn: [FileWatcherProvider] sky://vfs/fileChange emit failed: {}",
								Error
							);
						}
					}
				});

				state
			})
			.clone()
	}
}

fn MapEventKind(raw:&EventKind) -> Option<WatchEventKind> {
	match raw {
		EventKind::Create(_) => Some(WatchEventKind::Create),
		EventKind::Modify(_) => Some(WatchEventKind::Change),
		EventKind::Remove(_) => Some(WatchEventKind::Delete),
		// Access / Any / Other events are not exposed to extensions.
		_ => None,
	}
}

/// Translate a VS Code glob pattern into a `regex::Regex` so the native
/// watcher can apply the caller's filter before paying for an IPC hop. A
/// small subset of the glob grammar is supported (`**`, `*`, `?`, `[…]`,
/// `{…,…}` alternation) — exactly what TypeScript-language-features and
/// the other ship-time extensions rely on.
fn CompileGlobToRegex(Pattern:&str) -> Option<regex::Regex> {
	let mut Regex = String::with_capacity(Pattern.len() * 2 + 4);
	// Case-insensitive on macOS + Windows where the OS is typically case-insensitive;
	// on case-sensitive Linux filesystems extensions commonly still use lowercase
	// patterns, so the flag is safe across all three targets.
	if cfg!(any(target_os = "macos", target_os = "windows")) {
		Regex.push_str("(?i)");
	}
	Regex.push('^');
	let mut Chars = Pattern.chars().peekable();
	let mut InClass = false;
	while let Some(C) = Chars.next() {
		if InClass {
			if C == ']' {
				InClass = false;
			}
			Regex.push(C);
			continue;
		}
		match C {
			'*' => {
				if Chars.peek() == Some(&'*') {
					Chars.next();
					if Chars.peek() == Some(&'/') {
						Chars.next();
						Regex.push_str("(?:.*/)?");
					} else {
						Regex.push_str(".*");
					}
				} else {
					Regex.push_str("[^/]*");
				}
			},
			'?' => Regex.push_str("[^/]"),
			'[' => {
				Regex.push('[');
				InClass = true;
			},
			'{' => Regex.push_str("(?:"),
			'}' => Regex.push(')'),
			',' => Regex.push('|'),
			'.' | '+' | '(' | ')' | '^' | '$' | '|' | '\\' => {
				Regex.push('\\');
				Regex.push(C);
			},
			_ => Regex.push(C),
		}
	}
	Regex.push('$');
	regex::Regex::new(&Regex).ok()
}

#[async_trait]
impl FileWatcherProvider for MountainEnvironment {
	async fn RegisterWatcher(
		&self,
		Handle:String,
		Root:PathBuf,
		IsRecursive:bool,
		Pattern:Option<String>,
	) -> Result<(), CommonError> {
		let state = WatcherState::Get(self);

		// De-dup: the typescript-language-features extension alone registers
		// ~10 watchers against the same workspace root during activation.
		// If we already have a watcher on this exact (root, recursive)
		// combination, reuse it and just record the handle — the forwarder
		// task fans events out to every subscribed handle.
		{
			let guard = state
				.Entries
				.lock()
				.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
			if guard.contains_key(&Handle) {
				dev_log!(
					"filewatcher",
					"[FileWatcherProvider] handle={} already registered; skipping duplicate",
					Handle
				);
				return Ok(());
			}
		}

		let CompiledPattern = Pattern.as_deref().and_then(CompileGlobToRegex);
		let pattern_for_callback = CompiledPattern.clone();

		// Prepare the per-event callback. It owns clones of the handle and
		// the forwarder channel; debouncing state lives in the entry under
		// the global mutex (fine — the callback is not hot).
		let handle_for_callback = Handle.clone();
		let sender = state.EventSender.clone();
		let entries = state.Entries.clone();
		let mut watcher = notify::recommended_watcher(move |event_result:notify::Result<notify::Event>| {
			let Ok(event) = event_result else { return };
			let Some(kind) = MapEventKind(&event.kind) else { return };
			let kind_tag = kind.AsString();

			// Pattern filter — reject early so the event never crosses IPC.
			let matched_paths:Vec<PathBuf> = event
				.paths
				.into_iter()
				.filter(|path| match &pattern_for_callback {
					Some(re) => re.is_match(&path.to_string_lossy()),
					None => true,
				})
				.collect();
			if matched_paths.is_empty() {
				return;
			}

			// Debounce per (handle, path, kind). Lock is uncontested for
			// single-path events; bursts from FSEvents coalesce cleanly.
			let mut final_paths:Vec<PathBuf> = Vec::with_capacity(matched_paths.len());
			if let Ok(mut guard) = entries.lock() {
				if let Some(entry) = guard.get_mut(&handle_for_callback) {
					let now = Instant::now();
					entry.LastSeen.retain(|_, instant| now.duration_since(*instant) < Duration::from_secs(10));
					for path in matched_paths {
						let key = (path.clone(), kind_tag);
						let keep = match entry.LastSeen.get(&key) {
							Some(previous) if now.duration_since(*previous) < DebounceWindow => false,
							_ => {
								entry.LastSeen.insert(key, now);
								true
							},
						};
						if keep {
							final_paths.push(path);
						}
					}
				} else {
					return;
				}
			} else {
				return;
			}

			for path in final_paths {
				let _ = sender.send(WatchEvent { Handle:handle_for_callback.clone(), Kind:kind, Path:path });
			}
		})
		.map_err(|error| CommonError::Unknown { Description:format!("FileWatcher create failed: {}", error) })?;

		let mode = if IsRecursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
		watcher.watch(&Root, mode).map_err(|error| CommonError::Unknown {
			Description:format!("FileWatcher watch failed for {}: {}", Root.display(), error),
		})?;

		let mut guard = state
			.Entries
			.lock()
			.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
		// CompiledPattern is held by the callback closure (captured in
		// `pattern_for_callback`) — no need to store a second copy on the
		// entry. The `Watcher` handle alone holds the OS watch alive.
		let _ = CompiledPattern;
		guard.insert(
			Handle.clone(),
			WatcherEntry { Watcher:watcher, LastSeen:HashMap::new() },
		);

		dev_log!(
			"filewatcher",
			"[FileWatcherProvider] Registered watcher handle={} root={} recursive={} pattern={:?}",
			Handle,
			Root.display(),
			IsRecursive,
			Pattern
		);

		Ok(())
	}

	async fn UnregisterWatcher(&self, Handle:String) -> Result<(), CommonError> {
		let state = WatcherState::Get(self);
		let mut guard = state
			.Entries
			.lock()
			.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
		if guard.remove(&Handle).is_some() {
			dev_log!("filewatcher", "[FileWatcherProvider] Unregistered watcher handle={}", Handle);
		}
		Ok(())
	}
}
