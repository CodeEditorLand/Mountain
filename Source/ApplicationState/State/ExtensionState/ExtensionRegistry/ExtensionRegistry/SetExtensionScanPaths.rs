//! `ExtensionRegistry::SetExtensionScanPaths`

use super::Struct;
use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};
use tauri::Wry;
use crate::{Environment::CommandProvider::CommandHandler, dev_log};

pub fn Fn(This:&Struct, paths:Vec<PathBuf>) {
		if let Ok(mut guard) = This.ExtensionScanPaths.lock() {
			*guard = paths;
			dev_log!(
				"extensions",
				"[ExtensionRegistry] Extension scan paths updated ({} paths)",
				guard.len()
			);
		}
	}
