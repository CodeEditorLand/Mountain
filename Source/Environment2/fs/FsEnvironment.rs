/// @module FsEnvironment
/// @description The concrete environment for filesystem operations.
use Common::environment::Environment;
use tauri::{AppHandle, Wry};

#[derive(Clone)]
pub struct FsEnvironment {
	pub AppHandle:AppHandle<Wry>,
}

impl FsEnvironment {
	pub fn New(AppHandle:AppHandle<Wry>) -> Self { Self { AppHandle } }
}

impl Environment for FsEnvironment {}
