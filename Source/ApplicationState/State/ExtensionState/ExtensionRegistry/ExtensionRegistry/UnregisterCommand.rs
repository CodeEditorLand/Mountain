//! `ExtensionRegistry::UnregisterCommand`

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

pub fn Fn(This:&Struct, name:&str) {
		if let Ok(mut guard) = This.CommandRegistry.lock() {
			guard.remove(name);

			dev_log!("extensions", "[ExtensionRegistry] Command unregistered: {}", name);
		}
	}
