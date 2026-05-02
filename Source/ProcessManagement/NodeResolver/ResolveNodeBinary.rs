#![allow(non_snake_case)]

//! Resolve the Node binary used to spawn Cocoon and cache for the life of
//! the process. If everything fails returns `node` so `Command::new` still
//! tries a bare PATH lookup at spawn time, matching legacy behaviour while
//! logging the chain of misses.

use std::sync::OnceLock;

use tauri::{AppHandle, Runtime};

use crate::{
	ProcessManagement::NodeResolver::{CheckMinMajor, QueryNodeVersion, ResolveUncached, ResolvedNode},
	dev_log,
};

static RESOLVED:OnceLock<ResolvedNode::Struct> = OnceLock::new();

pub fn Fn<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> ResolvedNode::Struct {
	if let Some(Cached) = RESOLVED.get() {
		return Cached.clone();
	}

	let Resolved = ResolveUncached::Fn(ApplicationHandle);

	let Version = QueryNodeVersion::Fn(&Resolved.Path);
	match &Version {
		Some(Reported) => {
			dev_log!(
				"cocoon",
				"[NodeResolver] Using: {} (source={}, version={})",
				Resolved.Path.display(),
				Resolved.Source.AsLabel(),
				Reported
			);
			CheckMinMajor::Fn(Reported);
		},
		None => {
			dev_log!(
				"cocoon",
				"[NodeResolver] Using: {} (source={}, version=unknown)",
				Resolved.Path.display(),
				Resolved.Source.AsLabel()
			);
		},
	}

	// `OnceLock::set` is benign-racy: parallel callers resolve to the same
	// value; the first store wins.
	let _ = RESOLVED.set(Resolved.clone());

	Resolved
}
