//! `ExtensionRegistry::GetEnabledProposedAPIs`

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

pub fn Fn(This:&Struct) -> HashMap<String, Vec<String>> {
		This.EnabledProposedAPIs
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
