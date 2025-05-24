// ---------------------------------------------------------------------------------------------
// Mountain Protocol Handler (handlers/protocol.rs)
// --------------------------------------------------------------------------------------------
// Implements the handler for custom URI protocols registered by the
// application, primarily the `vscode://` (or potentially a custom `land://`)
// scheme. This allows external sources (e.g., OAuth redirects, CLI arguments,
// other apps) to trigger actions within the running Land instance.
//
// Responsibilities:
// - Implementing the callback function registered with Tauri's
//   `register_uri_scheme_protocol`.
// - Parsing the incoming URI (scheme, authority, path, query parameters) using
//   the `url` crate.
// - Routing the request based on the URI's authority and path.
// - Translating the request into an appropriate `ActionEffect` (e.g., opening a
//   file, executing a command, handling an auth callback).
// - Dispatching the effect asynchronously using the `AppRuntime`.
// - Returning an immediate, typically empty, HTTP response to the OS caller.
//
// Key Interactions:
// - Registered with and called by Tauri's protocol handling mechanism.
// - Uses the `url` crate for URI parsing.
// - Creates `ActionEffect`s (e.g., `workspace_effects`, `command_effects`).
// - Dispatches effects via `AppRuntime::run` (obtained via `AppHandle::state`).
// --------------------------------------------------------------------------------------------

// ----- START: Element/Mountain/src/handlers/protocol.rs -----
use std::{collections::HashMap, path::PathBuf, sync::Arc}; // Added PathBuf

use Land_Common::workspace_effects;
use Land_Common::{command_effects, effect::ActionEffect}; /* Effects for commands // Added ActionEffect //
                                                            * Added CommonError // Effects for opening files //
                                                            * Updated path // Updated path */
use serde_json::{Value, json};
use tauri::{
	AppHandle,
	Manager, // Added Manager trait for state access
	Runtime,
	api::ipc::{Request, Response},
	http::ResponseBuilder,
};
use url::Url;

use crate::runtime::AppRuntime; // To run effects // Use url crate for parsing

// Handle the registered vscode:// protocol
pub fn handle_vscode_protocol<R:Runtime>(
	request:&Request,
	app_handle:AppHandle<R>,
) -> Result<Response, Box<dyn std::error::Error>> {
	let uri_str = request.uri();
	println!("[Protocol Handler] Handling vscode URI: {}", uri_str);

	// --- Parse the URI ---
	// Use Url::parse and extract host (authority), path, query params
	match Url::parse(uri_str) {
		Ok(url) => {
			let authority = url.host_str().unwrap_or("").to_lowercase();
			let path = url.path().trim_start_matches('/');
			let query_params:HashMap<String, String> = url.query_pairs().into_owned().collect();

			println!(
				"[Protocol Handler] Authority='{}', Path='{}', Params='{:?}'",
				authority, path, query_params
			);

			// --- Route based on authority/path ---
			// Define the effect to be run asynchronously
			let effect_result:Result<ActionEffect<Arc<AppRuntime>, _, _>, String> = match authority.as_str() {
				"file" => {
					// Example: vscode://file/path/to/resource
					// Action: Open the file in the editor
					let file_path = PathBuf::from(path); // May need decoding? Ensure PathBuf is imported
					println!("[Protocol Handler] Request to open file: {}", file_path.display());
					// TODO: Create an effect to open a file/editor
					Ok(workspace_effects::open_file(file_path)) // Assume this effect exists
					// Err("open_file effect not implemented yet".to_string())
				},
				"vscode.git" | "vscode.github" | "vscode.vscode-remote" => {
					// Example: vscode://vscode.github/did-authenticate?code=...&state=...
					// Action: Handle authentication callback
					let command_id = format!("protocol.auth.{}", authority); // e.g., protocol.auth.github
					println!("[Protocol Handler] Request to handle auth callback for: {}", authority);
					// Pass relevant query params (code, state, error etc.) as args
					Ok(command_effects::execute_command(command_id, json!(query_params)))
				},
				"resource" => {
					// Example: vscode://resource/path/to/workspace/file
					// Treat similarly to 'file' for now
					let file_path = PathBuf::from(path);
					println!(
						"[Protocol Handler] Request to open resource (treating as file): {}",
						file_path.display()
					);
					Ok(workspace_effects::open_file(file_path))
					// Err("resource authority handling not fully
					// defined".to_string())
				},
				// Add other authorities like 'extension', 'settings', 'command' as needed
				_ => Err(format!("Unknown or unhandled authority: {}", authority)),
			};

			// --- Dispatch the Effect ---
			if let Ok(effect) = effect_result {
				let app_handle_clone = app_handle.clone();
				tokio::spawn(async move {
					// Need runtime access - best obtained via AppHandle state
					let runtime_state = app_handle_clone.try_state::<Arc<AppRuntime>>();
					if let Some(runtime) = runtime_state {
						println!("[Protocol Handler] Running effect for: {}", uri_str);
						if let Err(e) = runtime.run(effect).await {
							eprintln!("[Protocol Handler] Error running effect for {}: {}", uri_str, e);
						}
					} else {
						eprintln!("[Protocol Handler] AppRuntime state not found for effect execution!");
					}
				});
			} else {
				eprintln!(
					"[Protocol Handler] No effect created for URI: {}. Error: {}",
					uri_str,
					effect_result.unwrap_err()
				);
			}
		},
		Err(e) => {
			eprintln!("[Protocol Handler] Failed to parse vscode URI '{}': {}", uri_str, e);
		},
	}

	// --- Return Empty Response ---
	// The protocol handler usually just triggers an action asynchronously.
	ResponseBuilder::new().status(200).body(Vec::new())
}

// ----- END: Element/Mountain/src/handlers/protocol.rs -----
