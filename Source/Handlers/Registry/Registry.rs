

// File: Handlers/Registry/Registry.rs
// Defines a registry for mapping RPC method names to their corresponding handler functions.
// This provides a structured way to manage and dispatch incoming requests.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use log::{debug, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Runtime, Window};

use crate::Runtime::AppRuntime; // Assuming PascalCased

/// A type alias for a sidecar request handler function pointer.
/// This defines the expected signature for all registered RPC handlers.
pub type SidecarRequestHandlerFunction<R> = Arc<
    dyn Fn(
            AppHandle<R>,
            Window<R>,
            Arc<AppRuntime>,
            String, // Sidecar Identifier
            Value,  // Parameters
        ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

/// A registry for storing and retrieving sidecar request handlers.
pub struct HandlerRegistry<R: Runtime> {
    HandlerMap: HashMap<String, SidecarRequestHandlerFunction<R>>,
}

impl<R: Runtime> HandlerRegistry<R> {
    /// Creates a new, empty HandlerRegistry.
    pub fn New() -> Self {
        info!("[HandlerRegistry] New instance created.");
        Self { HandlerMap: HashMap::new() }
    }

    /// Registers a new handler for a specific RPC method.
    /// If a handler for the method already exists, it will be overwritten.
    pub fn Register<F>(
        &mut self,
        MethodName: &str,
        HandlerFunction: F,
    )
    where
        F: Fn(
                AppHandle<R>,
                Window<R>,
                Arc<AppRuntime>,
                String,
                Value,
            ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        if self.HandlerMap.contains_key(MethodName) {
            warn!("[HandlerRegistry] Overwriting existing handler for method '{}'", MethodName);
        }
        info!("[HandlerRegistry] Registering handler for method '{}'", MethodName);
        self.HandlerMap.insert(MethodName.to_string(), Arc::new(HandlerFunction));
    }

    /// Retrieves a handler for a specific RPC method.
    /// Returns `None` if no handler is registered for the given method name.
    pub fn Get(&self, MethodName: &str) -> Option<SidecarRequestHandlerFunction<R>> {
        match self.HandlerMap.get(MethodName) {
            Some(HandlerArc) => {
                debug!("[HandlerRegistry] Retrieved handler for method '{}'", MethodName);
                Some(HandlerArc.clone())
            }
            None => {
                debug!("[HandlerRegistry] No handler found for method '{}'", MethodName);
                None
            }
        }
    }
}

impl<R: Runtime> Default for HandlerRegistry<R> {
    /// Provides a default, empty `HandlerRegistry`.
    fn default() -> Self {
        Self::New()
    }
}

// File: Handlers/Registry/Registry.rs
// Defines a registry for mapping RPC method names to their corresponding handler functions.
// This provides a structured way to manage and dispatch incoming requests.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use log::{debug, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Runtime, Window};

use crate::Runtime::AppRuntime; // Assuming PascalCased

/// A type alias for a sidecar request handler function pointer.
/// This defines the expected signature for all registered RPC handlers.
pub type SidecarRequestHandlerFunction<R> = Arc<
    dyn Fn(
            AppHandle<R>,
            Window<R>,
            Arc<AppRuntime>,
            String, // Sidecar Identifier
            Value,  // Parameters
        ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

/// A registry for storing and retrieving sidecar request handlers.
pub struct HandlerRegistry<R: Runtime> {
    HandlerMap: HashMap<String, SidecarRequestHandlerFunction<R>>,
}

impl<R: Runtime> HandlerRegistry<R> {
    /// Creates a new, empty HandlerRegistry.
    pub fn New() -> Self {
        info!("[HandlerRegistry] New instance created.");
        Self { HandlerMap: HashMap::new() }
    }

    /// Registers a new handler for a specific RPC method.
    /// If a handler for the method already exists, it will be overwritten.
    pub fn Register<F>(
        &mut self,
        MethodName: &str,
        HandlerFunction: F,
    )
    where
        F: Fn(
                AppHandle<R>,
                Window<R>,
                Arc<AppRuntime>,
                String,
                Value,
            ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        if self.HandlerMap.contains_key(MethodName) {
            warn!("[HandlerRegistry] Overwriting existing handler for method '{}'", MethodName);
        }
        info!("[HandlerRegistry] Registering handler for method '{}'", MethodName);
        self.HandlerMap.insert(MethodName.to_string(), Arc::new(HandlerFunction));
    }

    /// Retrieves a handler for a specific RPC method.
    /// Returns `None` if no handler is registered for the given method name.
    pub fn Get(&self, MethodName: &str) -> Option<SidecarRequestHandlerFunction<R>> {
        match self.HandlerMap.get(MethodName) {
            Some(HandlerArc) => {
                debug!("[HandlerRegistry] Retrieved handler for method '{}'", MethodName);
                Some(HandlerArc.clone())
            }
            None => {
                debug!("[HandlerRegistry] No handler found for method '{}'", MethodName);
                None
            }
        }
    }
}

impl<R: Runtime> Default for HandlerRegistry<R> {
    /// Provides a default, empty `HandlerRegistry`.
    fn default() -> Self {
        Self::New()
    }
}
