//! `ExtensionRegistry::SetEnabledProposedAPIs`

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

pub fn Fn(This:&Struct, apis:HashMap<String, Vec<String>>) {
		if let Ok(mut guard) = This.EnabledProposedAPIs.lock() {
			*guard = apis;
			dev_log!(
				"extensions",
				"[ExtensionRegistry] Enabled proposed APIs updated ({} entries)",
				guard.len()
			);
		}
	}
