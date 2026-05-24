//! `Scheme::GetLandPort`

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

/// Get the port for a registered service
///
/// # Parameters
///
/// - `name`: Domain name to look up
///
/// # Returns
///
/// - `Some(port)` if service is registered
/// - `None` if service not found
pub fn Fn(name:&str) -> Option<u16> {
	let Registry = GetServiceRegistry()?;

	registry.Lookup(name).map(|S| s.port)
}
