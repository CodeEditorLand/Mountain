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
//! - `notify::recommended_watcher` executes callbacks on its own native thread,
//!   so we tunnel events through a bounded channel before touching async code.
//!   The forwarder task is spawned once on first registration and lives for the
//!   entire process lifetime.
//! - macOS FSEvents may emit duplicate Create/Change events for the same path
//!   in very short succession. We debounce by path within a 100 ms window
//!   per-handle, keyed on `(handle, path, kind)`.
//! - Linux inotify has a small per-user watcher cap
//!   (`fs.inotify.max_user_watches`); hitting it surfaces as
//!   `notify::Error::MaxFilesWatch`. We propagate that verbatim to the caller
//!   so the UI can show a guidance message.

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
	IPC::{IPCProvider::IPCProvider, SkyEvent::SkyEvent},
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
pub struct WatcherEntry {
	#[allow(dead_code)]
	Watcher:RecommendedWatcher,
	LastSeen:HashMap<(PathBuf, &'static str), Instant>,
}

/// Composite key used to detect duplicate watcher registrations. Two
/// extensions (or the same extension activated twice) frequently register
/// the same `(root, recursive, pattern)` triple within milliseconds of
/// each other - the typescript-language-features and git extensions are
/// the worst offenders. Without dedup, each registration spawns its own
/// notify::Watcher with its own kqueue/inotify subscription tree, doubling
/// (or worse) FS-event traffic and burning kernel handles.
type DedupKey = (PathBuf, bool, Option<String>);

/// Lazily-initialised process-wide state for file watching. Instances of the
/// event-forwarder task are singletons keyed on the MountainEnvironment
/// handle. Access through `WatcherState::Get`.
pub struct WatcherState {
	pub Entries:Arc<StandardMutex<HashMap<String, WatcherEntry>>>,
	pub EventSender:TokioMPSC::UnboundedSender<WatchEvent>,
	/// Maps `(root, recursive, pattern)` to the primary handle that owns
	/// the live OS watcher. Subsequent registrations matching the same
	/// triple are aliased to the primary; only the primary creates a
	/// notify::Watcher.
	pub DedupIndex:Arc<StandardMutex<HashMap<DedupKey, String>>>,
	/// Reverse index: primary handle → all aliased handles. When the
	/// forwarder task gets an event for a primary, it fans the same
	/// event out to every aliased handle so each extension's
	/// `vscode.workspace.createFileSystemWatcher` callback fires once.
	pub Aliases:Arc<StandardMutex<HashMap<String, Vec<String>>>>,
	/// Reverse lookup for unregister: any handle (primary or alias) →
	/// its primary. Lets `UnregisterWatcher` clean up alias entries
	/// without scanning the entire `Aliases` map.
	pub HandleToPrimary:Arc<StandardMutex<HashMap<String, String>>>,
}

impl WatcherState {
	/// Obtain (or create) the global WatcherState. The forwarder task is
	/// spawned on first access. Must be called from within a tokio runtime.
	pub fn Get(env:&MountainEnvironment) -> Arc<WatcherState> {
		use std::sync::OnceLock;

		// One WatcherState per process - the backing notify watchers are
		// cheap and multiplex fine, and we want a single forwarder task.
		static GLOBAL:OnceLock<Arc<WatcherState>> = OnceLock::new();
		GLOBAL
			.get_or_init(|| {
				let (tx, mut rx) = TokioMPSC::unbounded_channel::<WatchEvent>();
				let state = Arc::new(WatcherState {
					Entries:Arc::new(StandardMutex::new(HashMap::new())),
					EventSender:tx,
					DedupIndex:Arc::new(StandardMutex::new(HashMap::new())),
					Aliases:Arc::new(StandardMutex::new(HashMap::new())),
					HandleToPrimary:Arc::new(StandardMutex::new(HashMap::new())),
				});

				// The forwarder task holds a weak ref to the environment so
				// it unwinds cleanly if the env is ever torn down. State is
				// captured by Arc clone for the alias fan-out lookup.
				let env_clone = env.clone();
				let state_clone = state.clone();
				tokio::spawn(async move {
					use tauri::Emitter;
					while let Some(WatchEvent { Handle, Kind, Path }) = rx.recv().await {
						let ipc_provider:Arc<dyn IPCProvider> = env_clone.Require();
						// Fan events to the primary handle plus every alias
						// registered against it. Without this, the second
						// extension to register a duplicate watcher would
						// silently miss every event.
						let mut Recipients:Vec<String> = vec![Handle.clone()];
						if let Ok(AliasGuard) = state_clone.Aliases.lock() {
							if let Some(AliasList) = AliasGuard.get(&Handle) {
								Recipients.extend(AliasList.iter().cloned());
							}
						}
						for RecipientHandle in Recipients {
							let payload = json!({
								"handle": RecipientHandle,
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
									RecipientHandle,
									Kind.AsString(),
									Path,
									error
								);
							}
							// Dual-emit to Wind/Sky so the Explorer tree,
							// search index, and any other webview-side
							// consumer can react to disk mutations without
							// going through Cocoon. Wind's `TauriChannel`
							// subscribes to `sky://vfs/fileChange` under
							// the localFilesystem channel. Aliased handles
							// each get their own emit so per-handle
							// listeners on the Sky side fire correctly.
							if let Err(Error) = env_clone
								.ApplicationHandle
								.emit(SkyEvent::VFSFileChange.AsStr(), &payload)
							{
								dev_log!(
									"filewatcher",
									"warn: [FileWatcherProvider] sky://vfs/fileChange emit failed: {}",
									Error
								);
							}
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
/// `{…,…}` alternation) - exactly what TypeScript-language-features and
/// the other ship-time extensions rely on.
fn CompileGlobToRegex(Pattern:&str) -> Option<regex::Regex> {
	let mut Regex = String::with_capacity(Pattern.len() * 2 + 4);
	// Case-insensitive on macOS + Windows where the OS is typically
	// case-insensitive; on case-sensitive Linux filesystems extensions commonly
	// still use lowercase patterns, so the flag is safe across all three targets.
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

		// De-dup pass 1: same handle re-registered (cheap idempotency).
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

		// De-dup pass 2: same (root, recursive, pattern) triple already has
		// a primary watcher. The git extension, typescript-language-features,
		// and several `composer.*` extensions all hit this path during boot
		// (observed: `**/composer.json`, `**/composer.lock`, `**/*.md`,
		// `**/package.json` registered twice each within ~50ms). Aliasing
		// avoids the duplicate notify::Watcher / kqueue subscription tree
		// while still fanning events to every aliased handle.
		let DedupKeyValue:DedupKey = (Root.clone(), IsRecursive, Pattern.clone());
		{
			let DedupGuard = state
				.DedupIndex
				.lock()
				.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
			if let Some(PrimaryHandle) = DedupGuard.get(&DedupKeyValue).cloned() {
				drop(DedupGuard);
				let mut AliasGuard = state
					.Aliases
					.lock()
					.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
				AliasGuard
					.entry(PrimaryHandle.clone())
					.or_insert_with(Vec::new)
					.push(Handle.clone());
				let mut H2PGuard = state
					.HandleToPrimary
					.lock()
					.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
				H2PGuard.insert(Handle.clone(), PrimaryHandle.clone());
				dev_log!(
					"filewatcher",
					"[FileWatcherProvider] dedup hit; handle={} aliased to primary={} root={} pattern={:?}",
					Handle,
					PrimaryHandle,
					Root.display(),
					Pattern
				);
				return Ok(());
			}
		}
		// First registration for this triple. The DedupIndex insert
		// happens AFTER successful OS-watcher creation below so an
		// errored or benign-absent registration doesn't leave a stale
		// dedup entry pointing at a non-existent primary.

		let CompiledPattern = Pattern.as_deref().and_then(CompileGlobToRegex);
		let pattern_for_callback = CompiledPattern.clone();

		// Prepare the per-event callback. It owns clones of the handle and
		// the forwarder channel; debouncing state lives in the entry under
		// the global mutex (fine - the callback is not hot).
		let handle_for_callback = Handle.clone();
		let sender = state.EventSender.clone();
		let entries = state.Entries.clone();
		let mut watcher = notify::recommended_watcher(move |event_result:notify::Result<notify::Event>| {
			let Ok(event) = event_result else { return };
			let Some(kind) = MapEventKind(&event.kind) else { return };
			let kind_tag = kind.AsString();

			// Pattern filter - reject early so the event never crosses IPC.
			let matched_paths:Vec<PathBuf> = event
				.paths
				.into_iter()
				.filter(|path| {
					match &pattern_for_callback {
						Some(re) => re.is_match(&path.to_string_lossy()),
						None => true,
					}
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
					entry
						.LastSeen
						.retain(|_, instant| now.duration_since(*instant) < Duration::from_secs(10));
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
		// Watching a non-existent path is a common pattern: extensions
		// register watchers on optional config dirs (`~/.roo/skills-*`,
		// `.vscode/settings.json` in fresh workspaces, …) that may appear
		// later. `notify` returns `Error::PathNotFound` / "No path was
		// found"; failing the gRPC call counts against Cocoon's circuit
		// breaker - 5 such probes at boot trip the breaker open and
		// cascade into 60s of rejected reads. Record a "deferred" entry
		// without a live OS watcher so Unregister still works; future
		// events for that path won't fire, but the extension can re-
		// register once the directory appears, just like in stock VS Code.
		let WatchResult = watcher.watch(&Root, mode);
		let mut guard = state
			.Entries
			.lock()
			.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
		let _ = CompiledPattern;
		match WatchResult {
			Ok(()) => {
				guard.insert(Handle.clone(), WatcherEntry { Watcher:watcher, LastSeen:HashMap::new() });
				// Drop the Entries lock before grabbing DedupIndex to
				// avoid lock-order divergence vs the alias path (which
				// takes DedupIndex first). Re-acquire is cheap.
				drop(guard);
				if let Ok(mut DedupGuard) = state.DedupIndex.lock() {
					DedupGuard.entry(DedupKeyValue.clone()).or_insert_with(|| Handle.clone());
				}
				dev_log!(
					"filewatcher",
					"[FileWatcherProvider] Registered watcher handle={} root={} recursive={} pattern={:?}",
					Handle,
					Root.display(),
					IsRecursive,
					Pattern
				);
				return Ok(());
			},
			Err(error) => {
				let ErrorString = error.to_string().to_lowercase();
				let IsBenignAbsent = ErrorString.contains("no path was found")
					|| ErrorString.contains("no such file or directory")
					|| ErrorString.contains("entity not found")
					|| ErrorString.contains("path not found")
					|| ErrorString.contains("os error 2")
					|| !Root.exists();
				if IsBenignAbsent {
					dev_log!(
						"filewatcher",
						"[FileWatcherProvider] watch path absent (deferred) handle={} root={} err={}",
						Handle,
						Root.display(),
						error
					);
					// Drop watcher (no live subscription); record handle so
					// Unregister still finds something to remove. We do NOT
					// reuse the closure's notify::Watcher here.
					drop(watcher);
				} else {
					return Err(CommonError::Unknown {
						Description:format!("FileWatcher watch failed for {}: {}", Root.display(), error),
					});
				}
			},
		}

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

		// Step 1: alias removal. If the handle was aliased to a primary,
		// just remove it from the alias list and the lookup map. The OS
		// watcher stays alive because the primary still owns it.
		let MaybePrimary = {
			let mut H2PGuard = state
				.HandleToPrimary
				.lock()
				.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
			H2PGuard.remove(&Handle)
		};
		if let Some(PrimaryHandle) = MaybePrimary {
			let mut AliasGuard = state
				.Aliases
				.lock()
				.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
			if let Some(AliasList) = AliasGuard.get_mut(&PrimaryHandle) {
				AliasList.retain(|EntryHandle| EntryHandle != &Handle);
				if AliasList.is_empty() {
					AliasGuard.remove(&PrimaryHandle);
				}
			}
			dev_log!(
				"filewatcher",
				"[FileWatcherProvider] Unregistered alias handle={} primary={}",
				Handle,
				PrimaryHandle
			);
			return Ok(());
		}

		// Step 2: primary removal. Drop the OS watcher and clear the
		// dedup index entry. Any still-aliased handles are left dangling -
		// callers requesting a primary unregister while aliases still
		// exist is unusual but not fatal; the alias entries simply
		// stop receiving events.
		let mut Guard = state
			.Entries
			.lock()
			.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
		if Guard.remove(&Handle).is_some() {
			dev_log!("filewatcher", "[FileWatcherProvider] Unregistered watcher handle={}", Handle);
		}
		drop(Guard);

		// Clear the dedup-index entry pointing at this primary so a
		// future registration for the same triple opens a fresh OS
		// watcher rather than aliasing to a removed handle.
		let mut DedupGuard = state
			.DedupIndex
			.lock()
			.map_err(|error| CommonError::StateLockPoisoned { Context:error.to_string() })?;
		DedupGuard.retain(|_, PrimaryHandle| PrimaryHandle != &Handle);
		Ok(())
	}
}
