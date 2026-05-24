//! `ExtensionRegistry::RegisterCommand`

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

pub fn Fn(This:&Struct, name:String, handler:CommandHandler<Wry>) {
		if let Ok(mut guard) = This.CommandRegistry.lock() {
			guard.insert(name, handler);

			dev_log!("extensions", "[ExtensionRegistry] Command registered");
		}
	}
