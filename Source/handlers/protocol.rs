// ---------------------------------------------------------------------------------------------
// Mountain Protocol Handler (handlers/protocol.rs)
// --------------------------------------------------------------------------------------------
// Implements the handler for custom URI protocols registered by the
// application, primarily the `vscode://` (or a custom `landcode://`) scheme.
//
// Responsibilities:
// - Implementing the callback for Tauri's `register_uri_scheme_protocol`.
// - Parsing the incoming URI.
// - Routing the request based on URI authority/path.
// - Translating the request into an `ActionEffect`.
// - Dispatching the effect via `AppRuntime`.
// - Returning an immediate HTTP response to the OS.
//
// Key Interactions:
// - Registered with and called by Tauri.
// - Uses `url` crate.
// - Creates `ActionEffect`s.
// - Dispatches effects via `AppRuntime`.
// --------------------------------------------------------------------------------------------

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use Land_Common::{
	command_effects,

	effect::ActionEffect,

	errors::CommonError,

	workspace_effects,
	// If ui_effects::open_external_uri is added for non-file schemes
	// ui_effects,
};
use log::{debug, error, info, warn};
// Value is used for effect params
use serde_json::{Value, json};
use tauri::{
	AppHandle,

	Manager,

	Runtime,

	http::{Response as ResponseBuilder, StatusCode},
	// Renamed to avoid conflict
	ipc::{Request, Response as TauriResponse},
};
use url::Url;

// For consistent error formatting
use crate::{handlers::error_utils, runtime::AppRuntime};

pub fn handle_vscode_protocol<R:Runtime>(
	// request is &tauri::api::ipc::Request
	request:&Request,

	app_handle:AppHandle<R>,
) -> Result<TauriResponse, Box<dyn std::error::Error>> {
	// Returns a Tauri HTTP-like Response
	let uri_str = request.uri();

	info!("[Protocol Handler] Handling custom URI: {}", uri_str);

	match Url::parse(uri_str) {
		Ok(url) => {
			let authority = url.host_str().unwrap_or("").to_lowercase();

			// Percent-decoded path
			let path_str_from_url = url.path().to_string();

			let query_params:HashMap<String, String> = url.query_pairs().into_owned().collect();

			info!(
				"[Protocol Handler] Parsed: Authority='{}', Path='{}', Params='{:?}'",
				authority, path_str_from_url, query_params
			);

			// Define the effect to be run asynchronously.
			// The effect should generally produce a `Value` which can be `Value::Null` for
			// void ops.
			let effect_to_run_result:Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> = match authority
				.as_str()
			{
				"file" => {
					// Example: vscode://file//path/to/resource or vscode://file/C:/path/to/resource
					match url.to_file_path() {
						Ok(file_path) => {
							info!("[Protocol Handler] Request to open file: {}", file_path.display());

							Ok(workspace_effects::open_file(file_path))
						},

						Err(_) => {
							let err_msg =
								format!("Invalid file URI path component: '{}' for URI '{}'", url.path(), url.as_str());

							error!("[Protocol Handler] {}", err_msg);

							Err(error_utils::rpc_error_string(err_msg, Some("EBADURI_PATH")))
						},
					}
				},

				"vscode.open" | "open" => {
					// Handles vscode://vscode.open?uri=file%3A%2F%2F%2Fpath...
					// or vscode://open/file//path...
					let target_uri_to_open_str_opt = query_params
						.get("uri")
						.or_else(|| query_params.get("url"))
						.or_else(|| query_params.get("target"))
						.cloned();

					if let Some(target_uri_str) = target_uri_to_open_str_opt {
						match Url::parse(&target_uri_str) {
							Ok(parsed_target_uri) => {
								if parsed_target_uri.scheme() == "file" {
									match parsed_target_uri.to_file_path() {
										Ok(file_path) => {
											info!(
												"[Protocol Handler] 'vscode.open' (query) request for file: {}",
												file_path.display()
											);

											Ok(workspace_effects::open_file(file_path))
										},

										Err(_) => {
											Err(error_utils::rpc_error_string(
												format!(
													"Invalid file URI in 'vscode.open' args: {}",
													parsed_target_uri
												),
												Some("EBADURI_PARAM"),
											))
										},
									}
								} else {
									info!(
										"[Protocol Handler] 'vscode.open' (query) for non-file scheme '{}'. Currently \
										 unsupported for direct editor open.",
										parsed_target_uri.scheme()
									);

									// TODO: Potentially create an effect like
									// ui_effects::open_external_url(parsed_target_uri)
									Err(error_utils::rpc_error_string(
										format!(
											"Opening non-file URI '{}' via 'vscode.open' protocol not yet supported \
											 as an editor action.",
											parsed_target_uri
										),
										Some("ENOTSUP_SCHEME"),
									))
								}
							},

							Err(e) => {
								Err(error_utils::rpc_error_string(
									format!("Invalid URI in 'vscode.open' args ('{}'): {}", target_uri_str, e),
									Some("EBADURI_PARAM"),
								))
							},
						}
					} else if url.path_segments().map_or(false, |mut s| s.next() == Some("file")) {
						// Handle vscode://open/file//absolute/path or
						// vscode://vscode.open/file//absolute/path
						let mut segments = url.path_segments().expect("Path segments known to exist here");

						// Skip "file"
						segments.next();

						let path_after_file_segment = segments.collect::<Vec<_>>().join("/");

						// PathBuf::from needs care with leading slashes on Windows if it's not a UNC
						// path For `file:///C:/...` Url::path() gives `/C:/...`. PathBuf handles this.
						// For `file:////server/...` (UNC), Url::path() gives `//server/...`. PathBuf handles this.
						// For POSIX `/foo/bar`, Url::path() gives `/foo/bar`. PathBuf handles this.
						let file_path = PathBuf::from(path_after_file_segment);

						info!(
							"[Protocol Handler] 'vscode.open/file//' request for path: {}",
							file_path.display()
						);

						Ok(workspace_effects::open_file(file_path))
					} else {
						Err(error_utils::rpc_error_string(
							"Missing 'uri' query parameter or valid path structure for 'open' authority.".to_string(),
							Some("EBADARG_OPEN"),
						))
					}
				},

				"vscode.git" | "vscode.github" | "vscode.vscode-remote" | "landcode.auth" | "landcode.oauth" => {
					// Normalize authority for command name
					let command_name_suffix = authority.trim_start_matches("vscode.").trim_start_matches("landcode.");

					let command_id = format!("protocol.auth.{}", command_name_suffix);

					info!(
						"[Protocol Handler] Auth callback: Authority='{}', Command='{}'. Query: {:?}",
						authority, command_id, query_params
					);

					Ok(command_effects::execute_command(command_id, json!(query_params)))
				},

				"resource" => {
					// Example: vscode://resource/path/to/workspace/file
					match url.to_file_path() {
						// Assuming vscode://resource/ URIs that map to files
						Ok(file_path) => {
							info!("[Protocol Handler] Opening resource (as file): {}", file_path.display());

							Ok(workspace_effects::open_file(file_path))
						},

						Err(_) => {
							let err_msg = format!("Cannot convert 'resource' URI to file path: '{}'", url.as_str());

							error!("[Protocol Handler] {}", err_msg);

							Err(error_utils::rpc_error_string(err_msg, Some("EBADURI_RESOURCE")))
						},
					}
				},

				_ => {
					Err(error_utils::rpc_error_string(
						format!("Unknown or unhandled authority in custom URI: '{}'", authority),
						Some("EUNAUTH"),
					))
				},
			};

			if let Ok(effect) = effect_to_run_result {
				let app_handle_clone = app_handle.clone();

				// Clone for async task
				let uri_str_clone = uri_str.to_string();

				tauri::async_runtime::spawn(async move {
					// Use Tauri's async runtime
					if let Some(runtime) = app_handle_clone.try_state::<Arc<AppRuntime>>() {
						debug!("[Protocol Handler] Dispatching effect for URI: {}", uri_str_clone);

						if let Err(e) = runtime.run(effect).await {
							let err_str_for_log = error_utils::map_common_error_to_rpc_string(
								e,
								&format!("protocol_uri_effect_{}", uri_str_clone),
							);

							error!(
								"[Protocol Handler] Error running effect for URI {}: {}",
								uri_str_clone, err_str_for_log
							);

							// TODO: Potentially notify user via a UI message
							// effect if action failed and it's user-critical
						} else {
							info!("[Protocol Handler] Effect for URI {} completed successfully.", uri_str_clone);
						}
					} else {
						error!(
							"[Protocol Handler] AppRuntime state not found for URI action: {}",
							uri_str_clone
						);
					}
				});

				// Successfully dispatched an action (or attempted to), return 200 OK to OS.
				ResponseBuilder::builder().status(StatusCode::OK).body(Vec::new())
			} else {
				// This is an RPC error string
				let err_msg_json = effect_to_run_result.unwrap_err();

				error!(
					"[Protocol Handler] Failed to create effect for URI: {}. Error: {}",
					uri_str, err_msg_json
				);

				// If effect creation failed due to bad parameters from URI, return 400. If
				// unknown authority, 404. Inspecting the JSON string for codes is brittle.
				// Better to have structured error from effect_to_run_result if possible.
				// For now, assume most errors from effect_to_run_result mean bad input or
				// unhandled.
				let status_code = if err_msg_json.contains("EBADURI") || err_msg_json.contains("EBADARG") {
					StatusCode::BAD_REQUEST
				} else {
					// For EUNAUTH or other unhandled cases
					StatusCode::NOT_FOUND
				};

				ResponseBuilder::builder().status(status_code).body(err_msg_json.into_bytes())
			}
		},

		Err(e) => {
			// URI parsing failed
			let err_msg = format!("Failed to parse custom URI '{}': {}", uri_str, e);

			error!("[Protocol Handler] {}", err_msg);

			let response_body = error_utils::rpc_error_string(err_msg, Some("EBADURI_PARSE"));

			ResponseBuilder::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(response_body.into_bytes())
		},
	}
}
