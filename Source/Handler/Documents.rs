// ---------------------------------------------------------------------------------------------
// Mountain Document Handlers (handlers/documents.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests from Cocoon related to opening, creating, and saving
// documents, primarily by delegating the core logic to the DocumentProvider
// effect system. It also provides helper functions to notify Cocoon (via Vine)
// about document state changes initiated within Mountain.
//
// Responsibilities:
// - Handling document RPC calls by creating and dispatching
//   `documents_effects`.
// - Providing notification helpers to send `$accept...` notifications to
//   Cocoon.
//
// Key Interactions:
// - RPC handlers called by `track.rs` or `rpc.rs`.
// - Uses `documents_effects` and `AppRuntime`.
// - Notification helpers use `vine::send_notification_to_sidecar`.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

use Land_Common::documents_effects;
// For error mapping
use Land_Common::errors::CommonError;
use log::{error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use crate::{
	app_state::DocumentState,

	// Use shared error utilities
	handlers::error_utils,

	runtime::AppRuntime,

	vine,
};

// --- Helper: URI Parsing from Value ---

/// Parses a URI from a `serde_json::Value` representing URI components.
///
/// The `param_val` is expected to be a JSON object typically containing an
/// `external` field with the full URI string, or fallback to a `path` field
/// which might be an absolute file path or a generic URI string.
///
/// # Argument
/// * `param_val` - The `serde_json::Value` containing URI components.
/// * `method_name` - Name of the calling method, for error reporting.
/// * `arg_name` - Name of the argument being parsed, for error reporting.
/// * `arg_idx` - Optional index of the argument, for error reporting.
///
/// # Returns
/// * `Ok(Url)` if parsing is successful.
/// * `Err(String)` containing a JSON-RPC error string if parsing fails or
///   required fields are missing.
fn parse_uri_from_components_param(
	param_val:&Value,

	method_name:&str,

	arg_name:&str,

	arg_idx:Option<usize>,
) -> Result<Url, String> {
	// Attempt to get 'external' first, as it's usually the full URI string.
	let uri_str_opt = param_val.get("external").and_then(Value::as_str);

	let final_uri_str = uri_str_opt
		.map(String::from)
		.or_else(|| {
			// Fallback to 'path' if 'external' is not present or not a string.
			param_val.get("path").and_then(Value::as_str).map(|p_str| {
				// If 'path' looks like an absolute file path, try to convert it to a file URL.
				if PathBuf::from(p_str).is_absolute() {
					Url::from_file_path(p_str).map(|u| u.to_string()).unwrap_or_else(|e| {
						// If conversion fails, log and use the original path string.
						// This might happen if the path is malformed for a file URL.
						warn!(
							"[DocHandler URI Parse] Failed to convert absolute path '{}' to file URL: {}. Using as \
							 raw string.",
							p_str, e
						);

						p_str.to_string()
					})
				} else {
					// If 'path' is not absolute, assume it's already a scheme or an opaque URI
					// string.
					p_str.to_string()
				}
			})
		})
		.ok_or_else(|| {
			// If neither 'external' nor 'path' yields a usable string.
			error_utils::rpc_param_error_string(
				method_name,
				arg_name,
				"UriComponents DTO (with 'external' or 'path' string field)",
				arg_idx,
			)
		})?;

	Url::parse(&final_uri_str).map_err(|e| {
		error_utils::rpc_error_string(
			format!("Failed to parse URI '{}' in {}: {}", final_uri_str, method_name, e),
			Some("EBADURI"),
		)
	})
}

// --- Helper: lines and EOL from text ---
// Renamed from `lines_and_eol_from_text` for better clarity on its primary
// purpose. This function is also used by `app_state.rs` for
// `DocumentState::apply_changes`.

/// Splits text into lines and heuristically detects its End-Of-Line (EOL)
/// sequence.
///
/// Detection preference: CRLF > LF > CR.
/// If only CR is found, it's normalized to LF for internal consistency, similar
/// to how VS Code models often handle EOLs on load.
///
/// # Argument
/// * `text` - The input string to process.
///
/// # Returns
/// A tuple `(Vec<String>, String)` where the first element is a vector of lines
/// and the second is the detected (and possibly normalized) EOL string.
pub fn analyze_text_lines_and_eol(text:&str) -> (Vec<String>, String) {
	// Default to LF
	let mut detected_eol = "\n";

	if text.contains("\r\n") {
		detected_eol = "\r\n";
	} else if text.contains('\n') {
		// LF is already the default, this handles cases without CRLF.
		detected_eol = "\n";
	} else if text.contains('\r') {
		// Only CRs found. Normalize to LF for internal splitting and consistency.
		// VS Code's text model normalizes EOLs on load. Adopting a similar strategy.
		warn!(
			"[DocUtil] Text contains only CR EOLs. Normalizing to LF for internal processing. Text sample: '{}...'",
			text.chars().take(50).collect::<String>()
		);

		detected_eol = "\n";

		// Replace all CRs with LFs before splitting if we want to ensure split
		// works correctly for pure CR files. For this implementation, split
		// will still work because `detected_eol` is set to `\n`.
		// The current split will treat CRs as part of the line content if
		// `detected_eol` remains `\n`. If strict splitting of CR-only files
		// by CR is needed and then normalization: let lines_vec: Vec<String>
		// = text.split('\r').map(String::from).collect(); return (lines_vec,

		// Return normalized EOL
		// "\n".to_string());
	}

	// Splitting by the detected (or chosen normalized) EOL.
	// If `text` was purely CR and `detected_eol` became `\n`, this split might not
	// be what's intended if the goal was to split by CR first. However, since
	// we're normalizing pure CR to LF, splitting by `\n` (the normalized EOL)
	// after conceptually replacing CRs makes sense. For simplicity here, if only
	// CRs are found, we decide the EOL is '\n' and then split by '\n'. This means
	// the original CRs would be part of the line content if `text.replace('\r',

	// '\n')` isn't done. Let's refine: if detected_eol was set to \n due to
	// CR-only, we should split by original CR or normalize text first. The current
	// logic: if only CR, detected_eol becomes "\n". Then text.split("\n") is
	// called. This is fine if the goal is to treat the text as if it *had* LF
	// endings.

	let lines = text.split(detected_eol).map(String::from).collect();

	(lines, detected_eol.to_string())
}

// --- Handlers for RPC calls from Cocoon ---
// These handlers are typically invoked via `track.rs` when it maps an RPC
// method to an effect. The effect then calls the corresponding method on
// `DocumentProvider` (implemented in `environment.rs`), which in turn might
// call these specific handler functions if the logic isn't fully contained
// within the environment's DocumentProvider implementation.
// However, based on the file's description, these handlers seem to be the
// primary entry points from Track/RPC, and they themselves create and run the
// effects.

/// Handles the `$tryOpenDocument` RPC request from Cocoon.
///
/// This function orchestrates opening a document by running the
/// `documents_effects::try_open` effect.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array. The first element is expected to be
///   the `UriComponents` of the document to open.
///
/// # Returns
/// * `Ok(Value)` containing the `UriComponents` of the opened document.
/// * `Err(String)` with a JSON-RPC error if opening fails.
pub async fn handle_try_open_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	// Argument is UriComponents DTO
	let uri_components_val = args.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("$tryOpenDocument", "uriComponents", "Value::Object", Some(0))
	})?;

	info!(
		"[DocHandler] RPC $tryOpenDocument: URI(external)='{:?}'",
		uri_components_val.get("external")
	);

	trace!("[DocHandler] $tryOpenDocument full URI components: {:?}", uri_components_val);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect `documents_effects::try_open` expects:
	// 1. uri_components: Value (the DTO from Cocoon)
	// 2. language_id_opt: Option<String>
	// 3. content_opt: Option<String>
	// For opening an existing document, languageId and content are None.
	let effect = documents_effects::try_open(uri_components_val.clone(), None, None);

	runtime_state
		.run(effect)
		.await
		.map(|url_result| {
			// Construct UriComponents DTO for the response
			json!({




				// Standard marker for VS Code DTOs
				"$mid": 1,


				"scheme": url_result.scheme(),


				"path": url_result.path(),


				"external": url_result.to_string(),


				"fsPath": url_result.to_file_path().ok().as_ref().map_or_else(
					// Fallback for non-file URIs
					|| url_result.path(),


					|p| &p.to_string_lossy().into_owned()
				)
			})
		})
		.map_err(|e| {
			let op_context = format!("try_open_document for URI components: {:?}", uri_components_val);

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, &op_context)
		})
}

/// Handles the `$tryCreateDocument` RPC request from Cocoon.
///
/// This function creates a new, typically untitled, document by running the
/// `documents_effects::try_open` effect with appropriate parameters (e.g., null
/// URI, optional language and content).
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array. The first element (optional) is an
///   options object which may contain `language` and `content` fields.
///
/// # Returns
/// * `Ok(Value)` containing the `UriComponents` of the created document.
/// * `Err(String)` with a JSON-RPC error if creation fails.
pub async fn handle_try_create_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	// Options are optional, clone if present. Argument is an array: [options?]
	let options_val = args.get(0).cloned();

	info!("[DocHandler] RPC $tryCreateDocument: Options='{:?}'", options_val);

	let language_id_opt = options_val
		.as_ref()
		.and_then(|o| o.get("language"))
		.and_then(Value::as_str)
		.map(String::from);

	let content_opt = options_val
		.as_ref()
		.and_then(|o| o.get("content"))
		.and_then(Value::as_str)
		.map(String::from);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// For creating an untitled document, pass Value::Null as uri_components to
	// the try_open effect.
	let effect = documents_effects::try_open(Value::Null, language_id_opt, content_opt);

	runtime_state
		.run(effect)
		.await
		.map(|url_result| {
			// Construct UriComponents DTO for the response
			json!({




				"$mid": 1,


				"scheme": url_result.scheme(),


				"path": url_result.path(),


				"external": url_result.to_string(),


				"fsPath": url_result.to_file_path().ok().as_ref().map_or_else(
					|| url_result.path(),


					|p| &p.to_string_lossy().into_owned()
				)
			})
		})
		.map_err(|e| {
			let op_context = "try_create_document";

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, op_context)
		})
}

/// Handles the `$trySaveDocument` RPC request from Cocoon.
///
/// Saves an existing document by running the `documents_effects::try_save`
/// effect.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `uri_components_val` - A `serde_json::Value` representing the
///   `UriComponents` of the document to save. (Note: VS Code protocol usually
///   sends this as the first element of an array).
///
/// # Returns
/// * `Ok(Value::Bool)` indicating success (`true`) or failure (`false`) of the
///   save operation.
/// * `Err(String)` with a JSON-RPC error if saving fails.
pub async fn handle_try_save_document<R:Runtime>(
	app_handle:AppHandle<R>,

	// VS Code sends: [$trySaveDocument, [uriDto]]
	// So this `uri_components_val` is actually the `uriDto` from `params[0]`
	uri_components_val:Value,
) -> Result<Value, String> {
	info!(
		"[DocHandler] RPC $trySaveDocument: URI(external)='{:?}'",
		uri_components_val.get("external")
	);

	trace!("[DocHandler] $trySaveDocument full URI components: {:?}", uri_components_val);

	let uri = parse_uri_from_components_param(
		&uri_components_val,
		"$trySaveDocument",
		"uriComponents",
		// Assuming it's the first effective parameter
		Some(0),
	)?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::try_save(uri.clone());

	// The effect `try_save` returns `Result<bool, CommonError>`.
	// We need to map this to `Result<Value, String>`.
	runtime_state
		.run(effect)
		.await
		// Converts bool to Value::Bool
		.map(|success_bool| json!(success_bool))
		.map_err(|e| {

			let op_context = format!("try_save_document for {}", uri);


			error!("[DocHandler] Effect error for {}: {}", op_context, e);


			error_utils::map_common_error_to_rpc_string(e, &op_context)
		})
}

/// Handles the `$trySaveDocumentAs` RPC request from Cocoon.
///
/// Saves a document to a new location (or prompts the user for one) by running
/// the `documents_effects::try_save_as` effect.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `uri_components_val` - `UriComponents` of the original document. (Note: VS
///   Code protocol usually sends this as the first element of an array).
///
/// # Returns
/// * `Ok(Value)` which is either `UriComponents` of the newly saved document or
///   `Value::Null` if the user cancelled.
/// * `Err(String)` with a JSON-RPC error if the operation fails.
pub async fn handle_try_save_document_as<R:Runtime>(
	app_handle:AppHandle<R>,

	// VS Code sends: [$trySaveDocumentAs, [originalUriDto]]
	uri_components_val:Value,
) -> Result<Value, String> {
	info!(
		"[DocHandler] RPC $trySaveDocumentAs: Original URI(external)='{:?}'",
		uri_components_val.get("external")
	);

	trace!(
		"[DocHandler] $trySaveDocumentAs full original URI components: {:?}",
		uri_components_val
	);

	let original_uri = parse_uri_from_components_param(
		&uri_components_val,
		"$trySaveDocumentAs (original URI)",
		"uriComponents",
		// Assuming it's the first effective parameter
		Some(0),
	)?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect `try_save_as` with `None` for `new_target_uri` will trigger
	// Mountain's UI (via UiProvider) to pick a new path.
	let effect = documents_effects::try_save_as(original_uri.clone(), None);

	runtime_state
		.run(effect)
		.await
		.map(|new_uri_opt| {
			// Return null if user cancelled the save as dialog (effect returns Option<Url>)
			new_uri_opt.map_or(Value::Null, |new_uri| {
				json!({

					"$mid": 1,

					"scheme": new_uri.scheme(),

					"path": new_uri.path(),

					"external": new_uri.to_string(),

					"fsPath": new_uri.to_file_path().ok().as_ref().map_or_else(
						|| new_uri.path(),

						|p| &p.to_string_lossy().into_owned()
					)
				})
			})
		})
		.map_err(|e| {
			let op_context = format!("try_save_document_as for {}", original_uri);

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, &op_context)
		})
}

/// Handles the `$saveAll` RPC request from Cocoon.
///
/// Saves all dirty documents by running the `documents_effects::save_all`
/// effect.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `include_untitled` - Boolean indicating whether to attempt saving untitled
///   (in-memory) documents, which would typically prompt the user for a file
///   path.
///
/// # Returns
/// * `Ok(Value::Array(Vec<Value::Bool>))` where each boolean indicates success
///   of saving an individual document.
/// * `Err(String)` with a JSON-RPC error if the operation fails.
pub async fn handle_save_all<R:Runtime>(
	app_handle:AppHandle<R>,

	// VS Code sends: [$saveAll, includeUntitledBoolean]
	include_untitled:bool,
) -> Result<Value, String> {
	info!("[DocHandler] RPC $saveAll: includeUntitled={}", include_untitled);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::save_all(include_untitled);

	// The effect `save_all` returns `Result<Vec<bool>, CommonError>`.
	runtime_state
		.run(effect)
		.await
		// Converts Vec<bool> to Value::Array
		.map(|results_vec_bool| json!(results_vec_bool))
		.map_err(|e| {

			let op_context = "save_all";


			error!("[DocHandler] Effect error for {}: {}", op_context, e);


			error_utils::map_common_error_to_rpc_string(e, op_context)
		})
}

// --- Notification Helpers (Called by Mountain logic/effects, e.g.,

// DocumentProvider impl in environment.rs) ---
// These functions are responsible for sending notifications *to* Cocoon when
// document states change within Mountain.

/// Notifies Cocoon that a new document model has been added/opened.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`.
/// * `doc_state` - The `DocumentState` of the newly added document.
pub async fn notify_model_added<R:Runtime>(_:AppHandle<R>, doc_state:&DocumentState) {
	info!("[DocNotify] Sending $acceptModelAdded for: {}", doc_state.uri);

	trace!("[DocNotify] $acceptModelAdded state: {:?}", doc_state);

	let uri_components = json!({

		"$mid": 1,

		"scheme": doc_state.uri.scheme(),

		"path": doc_state.uri.path(),

		"external": doc_state.uri.to_string(),

		"fsPath": doc_state.uri.to_file_path().ok().as_ref().map_or_else(
			|| doc_state.uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptModelAdded(uri: UriComponents, eol: string, versionId:
	// number, lines: string[], languageId: string, isDirty: boolean, encoding:
	// string);

	// Note: VS Code also sends `isReadonly: boolean` which might be relevant if
	// Mountain supports readonly states.
	let payload = json!([
		uri_components,
		doc_state.eol,
		doc_state.version,
		doc_state.lines,
		doc_state.language_id,
		doc_state.is_dirty,
		// TODO: Ensure encoding is correctly determined and part of DocumentState.
		// Assuming this is part of DocumentState
		doc_state.encoding,
	]);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelAdded".to_string(), payload).await {
		error!("[DocNotify] Failed to send $acceptModelAdded for {}: {}", doc_state.uri, e);
	}
}

/// Notifies Cocoon that a document's content or state has changed.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (currently unused if vine is called
///   directly).
/// * `doc_uri` - URI of the changed document.
/// * `doc_version` - The new version ID of the document.
/// * `doc_eol` - The EOL sequence of the document.
/// * `doc_is_dirty` - The new dirty state of the document.
/// * `actual_changes_dto` - `serde_json::Value` representing an array of
///   `RpcModelContentChange` DTOs.
/// * `is_undoing` - True if the change is part of an undo operation.
/// * `is_redoing` - True if the change is part of a redo operation.
pub async fn notify_model_changed<R:Runtime>(
	// Not used if vine is called directly
	_app_handle:AppHandle<R>,

	doc_uri:&Url,

	doc_version:i64,

	doc_eol:&str,

	doc_is_dirty:bool,

	// Should be Vec<RpcModelContentChange> serialized as Value
	actual_changes_dto:Value,

	is_undoing:bool,

	is_redoing:bool,
) {
	info!("[DocNotify] Sending $acceptModelChanged V{} for: {}", doc_version, doc_uri);

	trace!(
		"[DocNotify] $acceptModelChanged changes DTO: {:?}, is_dirty: {}, is_undoing: {}, is_redoing: {}",
		actual_changes_dto, doc_is_dirty, is_undoing, is_redoing
	);

	let uri_components = json!({

		"$mid": 1,

		"scheme": doc_uri.scheme(),

		"path": doc_uri.path(),

		"external": doc_uri.to_string(),

		"fsPath": doc_uri.to_file_path().ok().as_ref().map_or_else(
			|| doc_uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// DTO structure based on VS Code's `IModelChangedEvent`
	// (src/vs/editor/common/model/textModelEvents.ts)
	let event_data_dto = json!({

		"versionId": doc_version,

		"changes": actual_changes_dto,

		"eol": doc_eol,

		"isUndoing": is_undoing,

		"isRedoing": is_redoing,

		// VS Code also sends:
		// Indicates a full replace, not granular changes.
		// isFlush?: boolean;

		// `actual_changes_dto` should reflect this if true.
	});

	// Protocol: $acceptModelChanged(uri: UriComponents, event:
	// IModelChangedEventDto, isDirty: boolean);

	let payload = json!([uri_components, event_data_dto, doc_is_dirty]);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelChanged for {}: {}", doc_uri, e);
	}
}

/// Notifies Cocoon that a document has been saved.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `uri` - URI of the saved document.
pub async fn notify_model_saved<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelSaved for: {}", uri);

	let uri_components = json!({
		"$mid": 1,

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"fsPath": uri.to_file_path().ok().as_ref().map_or_else(
			|| uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptModelSaved(uri: UriComponents);

	// Note: VS Code's DTO here is just `uri: UriComponents`.
	// The `$mid` and `fsPath` are added for consistency with how Mountain builds
	// UriComponents.
	// Send the UriComponents object directly
	let payload = json!(uri_components);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelSaved".to_string(), payload).await {
		error!("[DocNotify] Failed to send $acceptModelSaved for {}: {}", uri, e);
	}

	// Note: The responsibility to also send `notify_dirty_state_changed(...,

	// false)` after a successful save is handled by the `DocumentProvider`
	// implementation (e.g., in `environment.rs`'s `save_document` method),

	// which has the full context to decide if the dirty state actually
	// changed.
}

/// Notifies Cocoon that a document's dirty state has changed.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `uri` - URI of the document.
/// * `is_dirty` - The new dirty state.
pub async fn notify_dirty_state_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, is_dirty:bool) {
	info!("[DocNotify] Sending $acceptDirtyStateChanged ({}) for: {}", is_dirty, uri);

	let uri_components = json!({
		"$mid": 1,

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"fsPath": uri.to_file_path().ok().as_ref().map_or_else(
			|| uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptDirtyStateChanged(uri: UriComponents, isDirty: boolean);

	let payload = json!([uri_components, is_dirty]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptDirtyStateChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptDirtyStateChanged for {}: {}", uri, e);
	}
}

/// Notifies Cocoon that a document model has been removed/closed.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `uri` - URI of the removed document.
pub async fn notify_model_removed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelRemoved for: {}", uri);

	let uri_components = json!({
		"$mid": 1,

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"fsPath": uri.to_file_path().ok().as_ref().map_or_else(
			|| uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptModelRemoved(uri: UriComponents);

	// Send the UriComponents object directly
	let payload = json!(uri_components);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelRemoved".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelRemoved for {}: {}", uri, e);
	}
}

/// Notifies Cocoon that a document's language ID has changed.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `uri` - URI of the document.
/// * `language_id` - The new language ID.
pub async fn notify_language_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, language_id:String) {
	info!(
		"[DocNotify] Sending $acceptModelLanguageChanged ('{}') for: {}",
		language_id, uri
	);

	let uri_components = json!({
		"$mid": 1,

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"fsPath": uri.to_file_path().ok().as_ref().map_or_else(
			|| uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptModelLanguageChanged(uri: UriComponents, languageId:
	// string);

	let payload = json!([uri_components, language_id]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptModelLanguageChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelLanguageChanged for {}: {}", uri, e);
	}
}

/// Notifies Cocoon that a document's encoding has changed.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `uri` - URI of the document.
/// * `encoding` - The new encoding string (e.g., "utf8", "utf16le").
pub async fn notify_encoding_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, encoding:String) {
	info!("[DocNotify] Sending $acceptEncodingChanged ('{}') for: {}", encoding, uri);

	let uri_components = json!({
		"$mid": 1,

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"fsPath": uri.to_file_path().ok().as_ref().map_or_else(
			|| uri.path(),

			|p| &p.to_string_lossy().into_owned()
		)
	});

	// Protocol: $acceptEncodingChanged(uri: UriComponents, encoding: string);

	let payload = json!([uri_components, encoding]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptEncodingChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptEncodingChanged for {}: {}", uri, e);
	}
}

// NEW:
// // Example signature for a handler in handlers/documents.rs
// pub async fn handle_open_document_effect_logic<R: tauri::Runtime>(
//     app_handle: tauri::AppHandle<R>,
//     // Pass MountainEnvironment directly so the handler can call
// self.require::<FsReader>() etc.     // or if handlers are very thin, they
// might take Arc<FsReader>, Arc<UiProvider> etc. directly.     // For now,
// passing MountainEnvironment gives flexibility.
//     env: crate::environment::MountainEnvironment,
//     uri_components_dto: Value,
//     language_id_override_opt: Option<String>,
//     initial_content_opt: Option<String>,
// ) -> Result<Url, CommonError> {
//     // ... implementation using app_handle.state::<AppState>() ...
//     // ... and env.require::<Arc<dyn FsReader>>().read_file(...) ...
//     // ... or directly env.read_file(...) if FsReader is implemented on
// MountainEnvironment ...     // ... and calls to other
// handlers::documents::notify_* functions ...     todo!("Implement actual logic
// in handlers/documents.rs") }
