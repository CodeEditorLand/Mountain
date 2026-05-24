//! `FileWatcherProvider::Get`

use super::Struct;
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
use super::MountainEnvironment::Struct;
use crate::dev_log;

pub fn Fn(env:&MountainEnvironment) -> Arc<WatcherState> {
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
							let Payload = json!({
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
									"warn: [FileWatcherProvider] Failed to forward event handle={} kind={} path={:?}: \
									 {:?}",
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
							if let Err(Error) =
								env_clone.ApplicationHandle.emit(SkyEvent::VFSFileChange.AsStr(), &payload)
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
