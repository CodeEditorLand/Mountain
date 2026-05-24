//! `ExtensionRegistry::AddExtensionScanPath`

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

pub fn Fn(This:&Struct, path:PathBuf) {
		if let Ok(mut guard) = This.ExtensionScanPaths.lock() {
			guard.push(path.clone());

			dev_log!("extensions", "[ExtensionRegistry] Extension scan path added: {:?}", path);
		}
	}
