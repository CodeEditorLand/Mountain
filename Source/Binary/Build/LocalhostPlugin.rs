//! # Localhost Plugin Module
//!
//! Configures and creates the Tauri localhost plugin with CORS headers for Service Workers.

/// Creates and configures the localhost plugin with CORS headers preconfigured.
///
/// # Arguments
///
/// * `ServerPort` - The port number for the localhost server
///
/// # Returns
///
/// A configured `tauri_plugin_localhost::TauriPlugin` instance.
///
/// # CORS Configuration
///
/// The plugin is configured with permissive CORS headers to support Service Worker
/// and frontend integration:
/// - Access-Control-Allow-Origin: * (allows all origins)
/// - Access-Control-Allow-Methods: GET, POST, OPTIONS, HEAD
/// - Access-Control-Allow-Headers: Content-Type, Authorization, Origin, Accept
pub fn LocalhostPlugin(ServerPort: u16) -> tauri_plugin_localhost::TauriPlugin {
	tauri_plugin_localhost::Builder::new(ServerPort)
		.on_request(|_, Response| {
			// Set CORS headers to allow cross-origin requests from Service Workers
			Response.add_header("Access-Control-Allow-Origin", "*");
			Response.add_header(
				"Access-Control-Allow-Methods",
				"GET, POST, OPTIONS, HEAD",
			);
			Response.add_header(
				"Access-Control-Allow-Headers",
				"Content-Type, Authorization, Origin, Accept",
			);
		})
		.build()
}
