#![allow(non_snake_case)]

//! First-hit-wins ladder over the manager-specific lookups. Falls back to
//! `node` on PATH if every manager misses.

use std::path::PathBuf;

use tauri::{AppHandle, Runtime};

use crate::{
	ProcessManagement::NodeResolver::{
		NodeSource,
		ResolvedNode,
		TryAsdf,
		TryFnm,
		TryHomebrew,
		TryNvm,
		TryOverride,
		TryShipped,
		TryVolta,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(ApplicationHandle:&AppHandle<R>) -> ResolvedNode::Struct {
	if let Some(Found) = TryOverride::Fn() {
		return Found;
	}
	if let Some(Found) = TryShipped::Fn(ApplicationHandle) {
		return Found;
	}
	if let Some(Found) = TryFnm::Fn() {
		return Found;
	}
	if let Some(Found) = TryVolta::Fn() {
		return Found;
	}
	if let Some(Found) = TryAsdf::Fn() {
		return Found;
	}
	if let Some(Found) = TryNvm::Fn() {
		return Found;
	}
	if let Some(Found) = TryHomebrew::Fn() {
		return Found;
	}

	dev_log!(
		"cocoon",
		"[NodeResolver] No specific install found; falling back to `node` on PATH"
	);

	ResolvedNode::Struct { Path:PathBuf::from("node"), Source:NodeSource::Enum::Path }
}
