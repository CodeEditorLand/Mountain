// ---------------------------------------------------------------------------------------------
// Mountain Handler Registry  - POTENTIALLY
// DEPRECATED/UNUSED
// --------------------------------------------------------------------------------------------
// Defines a generic `HandlerRegistry` for mapping method names (strings) to
// asynchronous handler functions. This appears to be a foundational component
// for a flexible RPC or command dispatch system.
//
// **CURRENT STATUS: POTENTIALLY DEPRECATED OR AN ALTERNATIVE/EARLY CONCEPT.**
//
// The primary dispatch logic in Mountain seems to be handled by:
// 1. `track.rs`: For routing frontend commands and sidecar requests to effects
//    or specific handlers.
// 2. `rpc.rs`: For implementing `MainThread...Shape` RPC methods called by
//    sidecars, which are themselves invoked by `track.rs`.
//
// A generic registry like this might have been an initial approach or could be
// used for a different type of internal event or plugin system not yet fully
// realized. If it's not actively used by the main dispatch flow, its relevance
// should be clarified.
//
// This documentation describes its structure and potential usage as a generic
// handler mapping system.
//
// Responsibilities (if used):
// - Storing a map of method names to handler functions.
// - Providing methods to register and retrieve handlers.
// - Defining a standard signature for handler functions.
//
// Key Interactions (if used):
// - Would be populated during application setup.
// - A dispatcher would use `get()` to find and execute the appropriate handler
//   for a given method name.
// --------------------------------------------------------------------------------------------

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

// For logging, if registration/retrieval events are logged
use log::{debug, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Runtime, Window};

// Use the common AppRuntime, assuming handlers might need to execute effects.
use crate::runtime::AppRuntime;

// CommonError or a specific VineError might be relevant if this integrates with
// Example, if errors were Vine-specific
// Vine. use crate::vine::VineError;

/// Type alias for a sidecar request handler function.
///
/// A handler function is asynchronous and takes:
/// - `AppHandle<R>`: For accessing Tauri application resources and state.
/// - `Window<R>`: The main window context.
/// - `Arc<AppRuntime>`: For running effects.
/// - `String` (Sidecar ID): Identifier of the calling sidecar (if applicable).
/// - `Value` (Params): Parameters for the request.
/// It returns a `Result<Value, String>`, where `Ok(Value)` is the success
/// response and `Err(String)` is a JSON-RPC formatted error string.
pub type SidecarRequestHandlerFn<R> = Arc<
	// The handler must be `Send` and `Sync` to be stored and called across threads.
	// The returned future must also be `Send`.
	dyn Fn(
			AppHandle<R>,

			Window<R>,

			// For executing effects if needed
			Arc<AppRuntime>,

			// Originating sidecar_id or context identifier
			String,

			// Parameters for the handler
			Value,
		) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
		+ Send
		+ Sync,
>;

/// A generic registry for mapping method names to their handler functions.
///
/// This registry uses a `HashMap` to store handlers.
///
/// # Type Parameters
/// * `R`: A type that implements `tauri::Runtime`.
pub struct HandlerRegistry<R:Runtime> {
	handlers:HashMap<String, SidecarRequestHandlerFn<R>>,
}

impl<R:Runtime> HandlerRegistry<R> {
	/// Creates a new, empty `HandlerRegistry`.
	pub fn new() -> Self {
		info!("[Handler Registry] New instance created.");

		Self { handlers:HashMap::new() }
	}

	/// Registers a handler function for a specific method name.
	///
	/// If a handler for the given method name already exists, it will be
	/// overwritten.
	///
	/// # Argument
	/// * `method`: The method name (string slice) to associate with the
	///   handler.
	/// * `handler`: The handler function conforming to
	///   `SidecarRequestHandlerFn`'s underlying signature.
	pub fn register<F>(&mut self, method:&str, handler:F)
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
			// Ensure the handler function itself is 'static
			+ 'static, {
		if self.handlers.contains_key(method) {
			warn!("[Handler Registry] Overwriting existing handler for method '{}'", method);
		}

		info!("[Handler Registry] Registering handler for method '{}'", method);

		self.handlers.insert(method.to_string(), Arc::new(handler));
	}

	/// Retrieves a handler function for a given method name.
	///
	/// # Argument
	/// * `method`: The method name (string slice) whose handler is to be
	///   retrieved.
	///
	/// # Returns
	/// * `Some(SidecarRequestHandlerFn<R>)` if a handler is found.
	/// * `None` if no handler is registered for the method name.
	pub fn get(&self, method:&str) -> Option<SidecarRequestHandlerFn<R>> {
		match self.handlers.get(method) {
			Some(handler_arc) => {
				debug!("[Handler Registry] Retrieved handler for method '{}'", method);

				Some(handler_arc.clone())
			},

			None => {
				debug!("[Handler Registry] No handler found for method '{}'", method);

				None
			},
		}
	}

	// TODO: Consider adding an `unregister` method if dynamic unregistration is
	// needed. pub fn unregister(&mut self, method: &str) ->
	// Option<SidecarRequestHandlerFn<R>> {     info!("[Handler Registry]
	// Unregistering handler for method '{}'", method);     self.handlers.
	// remove(method) }

	// TODO: Consider adding a method to list all registered method names, useful
	// for debugging or introspection. pub fn list_methods(&self) -> Vec<String> {

	//     self.handlers.keys().cloned().collect()
	// }
}

impl<R:Runtime> Default for HandlerRegistry<R> {
	fn default() -> Self { Self::new() }
}

// --- Example Usage (Conceptual - How it might be used if integrated) ---
//
// In a setup function (e.g., in `main.rs` or a dedicated handlers module):
//
// ```rust
// fn initialize_custom_handlers<R: Runtime>(
//     app_handle: &AppHandle<R>,

//     runtime: Arc<AppRuntime>,

// registry: &mut HandlerRegistry<R> // If registry is managed state or passed
// around
//
// ) {

// This example assumes the registry is obtained or created here.
//
// In a real app, it might be part of AppState or managed by Tauri.
//
// Example instantiation
//     let mut registry = HandlerRegistry::<R>::new();

// Example handler registration
//
//     registry.register("custom_echo", |app, _win, _rt, _origin_id, params| {

//         Box::pin(async move {

//             info!("[Custom Echo Handler] Received echo request from '{}' with
// params: {:?}", _origin_id, params);

// Simple echo back
//
//             Ok(params)
//         })
//     });

//     registry.register("example_fs_stat", |app, win, rt, sid, params| {

// This would typically call a more specific handler from another module,

// e.g., handlers::native_fs::handle_fs_stat_deprecated(params)
//
// For demonstration, an inline mock:
//
//         Box::pin(async move {

//             info!("[Example FS Stat] Called by '{}', params: {:?}", sid,

// params);

// Mock implementation.
//
// In a real scenario, ensure `handlers::native_fs` functions match this
// signature
//
// or adapt the call.
//
//             Ok(json!({ "size": 1024, "type": "file" }))
//         })
//     });

// If the registry is part of AppState:
//
// let app_state = app_handle.state::<AppState>();

// let mut registry_guard = app_state.custom_handler_registry.lock().unwrap();

// registry_guard.register(...);

// If this registry becomes the primary dispatcher, it would need to be managed
// by Tauri.
//
// app_handle.manage(Arc::new(StdMutex::new(registry)));

// }

// In a dispatch function:
//
// async fn dispatch_custom_request<R: Runtime>(
//
//     app_handle: AppHandle<R>,

//     window: Window<R>,

//     runtime: Arc<AppRuntime>,

//     registry: Arc<StdMutex<HandlerRegistry<R>>>, // Accessed from managed
// state
//
//     method: String,

//     origin_id: String,

//     params: Value,

// ) -> Result<Value, String> {

//     let handler_registry_guard = registry.lock().unwrap();

//     if let Some(handler_fn) = handler_registry_guard.get(&method) {

//         handler_fn(app_handle, window, runtime, origin_id, params).await
//
//     } else {

//         Err(format!("No handler registered for method: {}", method))
//
//     }

// }

// ```
