//! `watch_file` gRPC endpoint - Cocoon calls this when an extension uses
//! `vscode.workspace.createFileSystemWatcher`. Routes to Mountain's
//! `FileWatcherProvider::RegisterWatcher` so OS events (FSEvents on macOS,
//! inotify on Linux) flow back to Cocoon as `$fileWatcher:event` gRPC
//! notifications which Cocoon fans out to extension `onDidChangeFile`
//! listeners.
//!
//! The proto `WatchFileRequest` only carries a `uri` field. We derive the
//! watch handle from a hash of the URI so dedup-by-triple logic in
//! `FileWatcherProvider` can collapse identical registrations from multiple
//! extensions watching the same root.

use std::{
	path::PathBuf,
	sync::atomic::{AtomicU64, Ordering},
};

use CommonLibrary::FileSystem::FileWatcherProvider::FileWatcherProvider;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, WatchFileRequest},
	dev_log,
};

static WATCH_SEQ:AtomicU64 = AtomicU64::new(1);

pub async fn Fn(Service:&CocoonServiceImpl, Request:WatchFileRequest) -> Result<Response<Empty>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("").to_string();

	if URI.is_empty() {
		return Ok(Response::new(Empty {}));
	}

	let Handle = format!("grpc-watch-{}", WATCH_SEQ.fetch_add(1, Ordering::Relaxed));

	dev_log!("filewatcher", "[CocoonService] watch_file handle={} uri={}", Handle, URI);

	let Root = if let Ok(Url) = url::Url::parse(&URI) {
		Url.to_file_path().unwrap_or_else(|_| PathBuf::from(&URI))
	} else {
		PathBuf::from(&URI)
	};

	// Register recursive with no pattern filter - Cocoon's FileSystemWatcher
	// subscribers apply their own glob matching on the extension side.
	Service
		.environment
		.RegisterWatcher(Handle, Root, true, None)
		.await
		.map_err(|E| Status::internal(format!("watch_file: {E}")))?;

	Ok(Response::new(Empty {}))
}
