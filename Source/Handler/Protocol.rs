// ---------------------------------------------------------------------------------------------
// Mountain Protocol Handler (handlers/protocol.rs)
// --------------------------------------------------------------------------------------------
// Implements the handler for custom URI protocols registered by the
// application, primarily for schemes like `vscode://` or a custom equivalent
// (e.g., `landcode://`). These URIs are typically invoked by the operating
// system when a user clicks a link associated with the application.
//
// Responsibilities:
// - Implementing the callback function for Tauri's
//   `register_uri_scheme_protocol`.
// - Parsing the incoming URI string into its components (authority, path, query
//   parameters).
// - Routing the request based on the URI's authority and path segments. Common
//   authorities include:
//   - `file`: To open a local file.
//   - `vscode.open` or `open`: To open a resource, often specified by a `uri`
//     query parameter.
//   - `vscode.git`, `vscode.github`, `vscode.vscode-remote`: Typically for
//     authentication callbacks or remote development scenarios.
//   - `resource`: Similar to `file`, for opening workspace resources.
// - Translating the parsed URI request into an appropriate `ActionEffect`
//   (e.g., `workspace_effects::open_file`, `command_effects::execute_command`).
// - Dispatching the created effect asynchronously via `AppRuntime`.
// - Returning an immediate HTTP-like response (e.g., 200 OK, 400 Bad Request)
//   to the operating system to acknowledge receipt of the URI. The actual
//   action happens asynchronously.
//
// Key Interactions:
// - Registered with Tauri during application setup
//   (`app.protocol().register(...)`).
// - Called by Tauri when the OS routes a custom scheme URI to the application.
// - Uses the `url` crate for robust URI parsing.
// - Creates `ActionEffect`s defined in `Land_Common`.
// - Dispatches these effects using `AppRuntime` (obtained from `AppHandle`).
// - Uses `handlers::error_utils` for formatting error messages in HTTP
//   responses.
// --------------------------------------------------------------------------------------------

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use Land_Common::{
	command_effects,

	// For creating ActionEffect instances
	effect::ActionEffect,

	errors::CommonError,

	workspace_effects,
	// If ui_effects::open_external_uri is added for non-file schemes, uncomment:
	// ui_effects,
};
use log::{debug, error, info, warn};
// Value is used for effect parameters (e.g., command arguments)
use serde_json::{Value, json};
use tauri::{
	AppHandle,

	// For app_handle.state(), app_handle.try_state()
	Manager,

	Runtime,

	// Renamed to avoid conflict
	http::{Response as TauriHttpResponseBuilder, StatusCode},

	// Tauri's IPC request/response types
	ipc::{Request as TauriIpcRequest, Response as TauriIpcResponse},
};
use url::Url;

// For consistent error formatting in the HTTP response body
use crate::{handlers::error_utils, runtime::AppRuntime};

/// Handles incoming custom URI scheme requests (e.g., `vscode://...`,
///
///
/// `landcode://...`).
///
/// This function is registered as the protocol handler with Tauri. It parses
/// the URI, determines the intended action based on its components, creates an
/// appropriate `ActionEffect`, and spawns an async task to run the effect using
/// the `AppRuntime`. It then returns an immediate HTTP response to the OS.
///
/// # Argument
/// * `request` - A `&tauri::ipc::Request` containing the full URI string.
/// * `app_handle` - The Tauri `AppHandle` for accessing `AppRuntime` and other
///   app resources.
///
/// # Returns
/// * `Result<TauriIpcResponse, Box<dyn std::error::Error>>` where
///   `TauriIpcResponse` is an HTTP-like response.
///   - `Ok` with a response (e.g., 200 OK) if the URI is accepted for
///     processing.
///   - `Ok` with an error response (e.g., 400 Bad Request, 404 Not Found) if
///     the URI is malformed or unhandled.
///   - `Err` for unexpected internal errors during response construction.
pub fn handle_custom_uri_scheme_request<R:Runtime>(
	// Note: Tauri now passes &Request for protocol handlers
	request:&TauriIpcRequest,

	app_handle:AppHandle<R>,
) -> Result<TauriIpcResponse, Box<dyn std::error::Error>> {
	let uri_str = request.uri();

	info!("[Protocol Handler] Received custom URI scheme request: {}", uri_str);

	match Url::parse(uri_str) {
		Ok(parsed_url) => {
			let authority = parsed_url.host_str().unwrap_or("").to_lowercase();

			// Path segments are already percent-decoded by `Url::path()`
			let path_from_url = parsed_url.path().to_string();

			// Query pairs are also percent-decoded
			let query_params:HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

			info!(
				"[Protocol Handler] Parsed URI: Authority='{}', Path='{}', QueryParams='{:?}'",
				authority, path_from_url, query_params
			);

			// Determine the ActionEffect to run based on URI components.
			// This returns a Result containing either the effect or a JSON error string.
			let effect_to_run_result:Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> = match authority
				.as_str()
			{
				"file" => {
					// Handles URIs like:
					// - `vscode://file//path/to/resource` (POSIX)
					// - `vscode://file/C:/path/to/resource` (Windows)
					// The `Url::to_file_path()` method correctly handles these variations.
					match parsed_url.to_file_path() {
						Ok(file_path) => {
							info!(
								"[Protocol Handler] 'file' authority: Request to open file: {}",
								file_path.display()
							);

							Ok(workspace_effects::open_file(file_path))
						},

						Err(_) => {
							// This error occurs if the path part of a "file://" URI is not a valid
							// file path for the current OS (e.g., invalid characters,

							// wrong format).
							let err_msg = format!(
								"Invalid file path component in URI '{}' for 'file' authority: path part was '{}'",
								parsed_url.as_str(),
								parsed_url.path()
							);

							error!("[Protocol Handler] {}", err_msg);

							Err(error_utils::rpc_error_string(err_msg, Some("EBADURI_PATH")))
						},
					}
				},

				"vscode.open" | "open" => {
					// Handles URIs like:
					// - `vscode://vscode.open?uri=file%3A%2F%2F%2Fpath...`
					// - `vscode://open/file//path/to/resource`
					// - `vscode://open?target=http%3A%2F%2Fexample.com`

					// Prioritize 'uri' query parameter, then 'url', then 'target'.
					let target_uri_to_open_str_opt = query_params
						.get("uri")
						.or_else(|| query_params.get("url"))
						.or_else(|| query_params.get("target"))
						.cloned();

					if let Some(target_uri_str) = target_uri_to_open_str_opt {
						// Case 1: Target URI is in a query parameter.
						match Url::parse(&target_uri_str) {
							Ok(parsed_target_uri) => {
								if parsed_target_uri.scheme() == "file" {
									match parsed_target_uri.to_file_path() {
										Ok(file_path) => {
											info!(
												"[Protocol Handler] '{}.open' (query): Opening file: {}",
												// "vscode.open" or "open"
												authority,
												file_path.display()
											);

											Ok(workspace_effects::open_file(file_path))
										},

										Err(_) => {
											Err(error_utils::rpc_error_string(
												format!(
													"Invalid file URI in '{}.open' query parameter: {}",
													authority, parsed_target_uri
												),
												Some("EBADURI_PARAM"),
											))
										},
									}
								} else {
									// TODO: Handle non-file schemes for 'vscode.open'.
									//       This might involve opening in an external browser or a webview.
									//       Example: Ok(ui_effects::open_external_url(parsed_target_uri))
									warn!(
										"[Protocol Handler] '{}.open' (query): Opening non-file URI '{}' is currently \
										 unsupported for direct editor action. Consider ui_effects::open_external_url.",
										authority, parsed_target_uri
									);

									Err(error_utils::rpc_error_string(
										format!(
											"Opening non-file URI '{}' via '{}.open' protocol as an editor action is \
											 not yet supported.",
											parsed_target_uri, authority
										),
										Some("ENOTSUP_SCHEME"),
									))
								}
							},

							Err(e) => {
								Err(error_utils::rpc_error_string(
									format!(
										"Invalid URI in '{}.open' query parameter ('{}'): {}",
										authority, target_uri_str, e
									),
									Some("EBADURI_PARAM"),
								))
							},
						}
					} else if parsed_url.path_segments().map_or(false, |mut s| s.next() == Some("file")) {
						// Case 2: Path-style URI like `vscode://open/file//absolute/path`
						// `parsed_url.path()` would be `/file//absolute/path`
						// We need to extract the actual file path after "/file/"
						let path_after_file_segment = parsed_url
							.path()
							.strip_prefix("/file/")
							.map(|s| {
								// On Windows, paths might start with `/C:/...`. `PathBuf` handles this.
								// For UNC paths `//server/share`, they appear as `/server/share` here.
								if cfg!(windows) && s.starts_with('/') && s.len() > 1 && s.as_bytes()[1] == b':' {
									// Path like /C:/... , remove leading / for PathBuf on windows
									&s[1..]
								} else {
									s
								}
							})
							.unwrap_or("");

						if path_after_file_segment.is_empty() {
							Err(error_utils::rpc_error_string(
								"Missing file path after '/file/' segment in 'open' authority URI.".to_string(),
								Some("EBADURI_PATH_OPEN"),
							))
						} else {
							let file_path = PathBuf::from(path_after_file_segment);

							info!(
								"[Protocol Handler] '{}.open/file//': Opening path: {}",
								authority,
								file_path.display()
							);

							Ok(workspace_effects::open_file(file_path))
						}
					} else {
						Err(error_utils::rpc_error_string(
							format!(
								"Missing 'uri' query parameter or valid '/file/...' path structure for '{}' authority.",
								authority
							),
							Some("EBADARG_OPENURI"),
						))
					}
				},

				// Authorities related to authentication or specific integrations
				"vscode.git" | "vscode.github" | "vscode.vscode-remote" | "landcode.auth" | "landcode.oauth" => {
					// Normalize authority for command name generation (e.g., "vscode.git" -> "git")
					let command_suffix = authority.trim_start_matches("vscode.").trim_start_matches("landcode.");

					let command_id = format!("protocol.auth.{}", command_suffix);

					info!(
						"[Protocol Handler] Auth callback: Authority='{}', mapped to Command='{}'. QueryParams: {:?}",
						authority, command_id, query_params
					);

					// Pass all query parameters as arguments to the command
					Ok(command_effects::execute_command(command_id, json!(query_params)))
				},

				"resource" => {
					// Handles URIs like `vscode://resource/path/to/workspace/file`
					// This is often used for resources within the workspace or extension resources.
					// For MVP, assume these map to local file paths.
					match parsed_url.to_file_path() {
						Ok(file_path) => {
							info!(
								"[Protocol Handler] 'resource' authority: Opening as file: {}",
								file_path.display()
							);

							Ok(workspace_effects::open_file(file_path))
						},

						Err(_) => {
							let err_msg = format!(
								"Cannot convert 'resource' URI to a local file path: '{}'",
								parsed_url.as_str()
							);

							error!("[Protocol Handler] {}", err_msg);

							Err(error_utils::rpc_error_string(err_msg, Some("EBADURI_RESOURCE")))
						},
					}
				},

				_ => {
					// Unknown authority
					let err_msg = format!("Unknown or unhandled authority in custom URI: '{}'", authority);

					warn!("[Protocol Handler] {}", err_msg);

					Err(error_utils::rpc_error_string(err_msg, Some("EUNAUTH")))
				},
			};

			// --- Dispatch Effect Asynchronously and Return HTTP Response ---
			if let Ok(effect) = effect_to_run_result {
				let app_handle_clone = app_handle.clone();

				// Clone for async task
				let uri_str_clone = uri_str.to_string();

				// Spawn a new async task to run the effect.
				// This allows the protocol handler to return an immediate HTTP response.
				tauri::async_runtime::spawn(async move {
					// Get AppRuntime from Tauri's managed state.
					if let Some(runtime_state) = app_handle_clone.try_state::<Arc<AppRuntime>>() {
						// Clone Arc<AppRuntime>
						let runtime = runtime_state.inner().clone();

						debug!("[Protocol Handler Task] Dispatching effect for URI: {}", uri_str_clone);

						if let Err(e) = runtime.run(effect).await {
							// Log the error from effect execution.
							let err_str_for_log = error_utils::map_common_error_to_rpc_string(
								e,
								&format!("protocol_uri_effect_failed_{}", uri_str_clone),
							);

							error!(
								"[Protocol Handler Task] Error running effect for URI {}: {}",
								uri_str_clone, err_str_for_log
							);

							// TODO: Consider notifying the user via a UI
							// message effect if the       action was
							// user-initiated and critical, and it failed.
							//       Example:
							// runtime.run(ui_effects::show_message(Error,

							// ...)).await;
						} else {
							info!(
								"[Protocol Handler Task] Effect for URI {} completed successfully.",
								uri_str_clone
							);
						}
					} else {
						error!(
							"[Protocol Handler Task] AppRuntime state not found for URI action: {}. Effect not run.",
							uri_str_clone
						);
					}
				});

				// Successfully parsed and an action is being dispatched. Return 200 OK.
				// Empty body for 200
				TauriHttpResponseBuilder::new().status(StatusCode::OK).body(Vec::new())
			} else {
				// Effect creation failed (e.g., bad URI parameters, unhandled authority).
				// This is already a JSON string.
				let err_msg_json_str = effect_to_run_result.unwrap_err();

				error!(
					"[Protocol Handler] Failed to create effect for URI: {}. Error: {}",
					uri_str, err_msg_json_str
				);

				// Determine appropriate HTTP status code based on the error.
				let status_code = if err_msg_json_str.contains("EBADURI") || err_msg_json_str.contains("EBADARG") {
					StatusCode::BAD_REQUEST
				} else if err_msg_json_str.contains("EUNAUTH") || err_msg_json_str.contains("ENOTSUP") {
					// Or NOT_IMPLEMENTED (501) for ENOTSUP
					StatusCode::NOT_FOUND
				} else {
					// Default for other errors
					StatusCode::INTERNAL_SERVER_ERROR
				};

				TauriHttpResponseBuilder::new()
					.status(status_code)
					.body(err_msg_json_str.into_bytes())
			}
		},

		Err(e) => {
			// URI parsing itself failed.
			let err_msg = format!("Failed to parse custom URI '{}': {}", uri_str, e);

			error!("[Protocol Handler] {}", err_msg);

			let response_body_json_str = error_utils::rpc_error_string(err_msg, Some("EBADURI_PARSE"));

			TauriHttpResponseBuilder::new()
				.status(StatusCode::BAD_REQUEST)
				.body(response_body_json_str.into_bytes())
		},
	}
}
