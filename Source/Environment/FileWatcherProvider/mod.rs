pub mod Get;

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

/// Internal entry tracked per registered watcher. The `Watcher` handle must
/// be kept alive for the lifetime of the registration; dropping it releases
/// the OS resources.
pub struct WatcherEntry {
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

#[derive(Debug, Clone)]
pub struct Struct;
