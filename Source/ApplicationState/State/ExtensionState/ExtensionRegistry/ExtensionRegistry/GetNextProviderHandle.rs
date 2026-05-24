//! `ExtensionRegistry::GetNextProviderHandle`

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

pub fn Fn(This:&Struct) -> u32 { This.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }
