// @module FileSystemEnvironment
// @description The concrete environment for filesystem operations.
use Common::environment::Environment;
use tauri::{ApplicationHandle, Wry};

#[derive(Clone)]
pub struct FileSystemEnvironment {
	pub ApplicationHandle:ApplicationHandle<Wry>,
}

impl FileSystemEnvironment {
	pub fn New(ApplicationHandle:ApplicationHandle<Wry>) -> Self { Self { ApplicationHandle } }
}

impl Environment for FileSystemEnvironment {}
