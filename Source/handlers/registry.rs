// Mountain/src/handlers/registry.rs
use crate::runtime::AppRuntime; // Use the common runtime
use crate::vine::VineError;
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tauri::{AppHandle, Runtime, Window}; // Use common error type

// Define the signature for a handler function
pub type SidecarRequestHandler<R> = Arc<
    dyn Fn(
            AppHandle<R>,
            Window<R>,
            Arc<AppRuntime>,
            String,
            Value,
        ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

// Simple registry using a HashMap
pub struct HandlerRegistry<R: Runtime> {
    handlers: HashMap<String, SidecarRequestHandler<R>>,
}

impl<R: Runtime> HandlerRegistry<R> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    // Register a handler for a specific method name
    pub fn register<F>(&mut self, method: &str, handler: F)
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
        println!("[Handler Registry] Registering handler for '{}'", method);
        self.handlers.insert(method.to_string(), Arc::new(handler));
    }

    // Get a handler for a method name
    pub fn get(&self, method: &str) -> Option<SidecarRequestHandler<R>> {
        self.handlers.get(method).cloned()
    }
}

// --- Example Registration in main.rs or handlers module ---
// fn register_handlers<R: Runtime>(registry: &mut HandlerRegistry<R>) {
//     registry.register("fs_stat", |app, win, rt, sid, params| Box::pin(handlers::native_fs::handle_fs_stat(app, win, rt, sid, params)));
//     registry.register("config_getConfiguration", |app, win, rt, sid, params| Box::pin(handlers::config::handle_get_configuration(app, win, rt, sid, params)));
//     // ... register all other handlers ...
// }
