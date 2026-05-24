//! `Scheme::InitServiceRegistry`

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

/// Initialize the global service registry
///
/// This must be called once during application setup before any land://
/// requests.
pub fn Fn(registry:ServiceRegistry) {
	let mut registry_lock = SERVICE_REGISTRY.write().unwrap();

	*registry_lock = Some(registry);
}
