//! `ExtensionRegistry::EnableProposedAPI`

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

pub fn Fn(This:&Struct, ExtensionId:String, api_name:String) {
		if let Ok(mut guard) = This.EnabledProposedAPIs.lock() {
			guard.entry(ExtensionId).or_insert_with(Vec::new).push(api_name);

			dev_log!("extensions", "[ExtensionRegistry] Proposed API enabled");
		}
	}
