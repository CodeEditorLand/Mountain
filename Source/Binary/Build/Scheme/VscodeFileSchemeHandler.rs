//! `Scheme::VscodeFileSchemeHandler`

use std::{
	collections::HashMap,
	panic::{AssertUnwindSafe, catch_unwind},
	sync::RwLock,
};
use tauri::http::{
	Method,
	request::Request,
	response::{Builder, Response},
};
use super::ServiceRegistry::Struct;
use crate::dev_log;

static SERVICE_REGISTRY:RwLock<Option<ServiceRegistry>> = RwLock::new(None);
static CACHE:RwLock<Option<HashMap<String, CacheEntry>>> = RwLock::new(None);

/// Handles `vscode-file://` custom protocol requests.
///
/// VS Code's Electron workbench computes asset URLs as:
///   `vscode-file://vscode-app/{appRoot}
