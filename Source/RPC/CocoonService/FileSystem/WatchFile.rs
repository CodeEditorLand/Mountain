#![allow(non_snake_case)]

//! Register a file watcher. TODO(P1): wire `notify` and store the
//! handle in `ApplicationState.Feature.Watchers` so `cancel_operation`
//! can stop it.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, WatchFileRequest},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:WatchFileRequest) -> Result<Response<Empty>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	dev_log!(
		"cocoon",
		"[CocoonService] watch_file registered (polling not yet active): {}",
		URI
	);

	Ok(Response::new(Empty {}))
}
