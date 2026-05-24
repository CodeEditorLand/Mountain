//! `Scheme::RegisterLandService`

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

/// Register a service with the land:// scheme
///
/// This helper function makes it easy to register local services.
///
/// # Parameters
///
/// - `name`: Domain name (e.g., "code.land.playform.cloud")
/// - `port`: Local port where the service is listening
pub fn Fn(name:&str, port:u16) {
	let Registry = GetServiceRegistry().expect("Service registry not initialized. Call InitServiceRegistry first.");

	registry.Register(name.to_string(), port, Some("/health".to_string()));

	dev_log!("lifecycle", "[Scheme] Registered service: {} -> {}", name, port);
}
