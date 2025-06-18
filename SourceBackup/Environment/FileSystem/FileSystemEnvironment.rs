// @module FileSystemEnvironment
// @description The concrete Environment for filesystem operations.
// NOTE: This file is part of a legacy structure. The main implementation
// is now in `MountainEnvironment` which directly implements the provider
// traits.

#![allow(non_snake_case)]

use Common::Environment::Environment;
use tauri::{AppHandle, Wry};

#[derive(Clone)]
pub struct FileSystemEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,
}

impl FileSystemEnvironment {
	pub fn New(app_handle:AppHandle<Wry>) -> Self { Self { ApplicationHandle:app_handle } }
}

impl Environment for FileSystemEnvironment {}
