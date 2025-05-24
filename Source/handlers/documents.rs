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
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State};
use url::Url;

use crate::{
	app_state::{AppState, DocumentState},

	// Use shared error utilities
	handlers::error_utils,

	runtime::AppRuntime,

	vine,
};

// --- Helper: URI Parsing from Value ---
fn parse_uri_from_components_param(
	param_val:&Value,

	method_name:&str,

	arg_name:&str,

	arg_idx:Option<usize>,
) -> Result<Url, String> {
	// Fallback to path if external is missing
	let uri_str = param_val
		.get("external")
		.and_then(Value::as_str)
		.or_else(|| {
			param_val.get("path").and_then(Value::as_str).map(|p_str| {
				// If path is absolute, try to form file URL

				if PathBuf::from(p_str).is_absolute() {
					Url::from_file_path(p_str)
						.map(|u| u.to_string())
						.unwrap_or_else(|_| p_str.to_string())
				// Otherwise, assume it's a scheme or opaque URI string
				} else {
					p_str.to_string()
				}
			})
		})
		.ok_or_else(|| {
			error_utils::rpc_param_error_string(method_name, arg_name, "UriComponents ({external} or {path})", arg_idx)
		})?;

	Url::parse(uri_str).map_err(|e| {
		error_utils::rpc_error_string(
			format!("Failed to parse URI '{}' in {}: {}", uri_str, method_name, e),
			Some("EBADURI"),
		)
	})
}

// --- Helper: lines and EOL from text (public for app_state if needed there
// too) ---
/// Utility to split text into lines and detect its EOL sequence.
pub fn lines_and_eol_from_text(text:&str) -> (Vec<String>, String) {
	// Default to LF
	let mut detected_eol = "\n";

	if text.contains("\r\n") {
		detected_eol = "\r\n";

	// Check after \r\n
	} else if text.contains('\n') {
		detected_eol = "\n";

	// Check for lone \r only if others are not present
	} else if text.contains('\r') {
		// For lone \r, standardizing to \n for internal consistency and splitting.

		// VS Code model normalizes EOLs on load.

		detected_eol = "\n";
	}

	// Splitting by the detected EOL.

	let lines = text.split(detected_eol).map(String::from).collect();

	(lines, detected_eol.to_string())
}

// --- Handlers for RPC calls from Cocoon ---

pub async fn handle_try_open_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let uri_components = args
		.get(0)
		.ok_or_else(|| error_utils::rpc_param_error_string("$tryOpenDocument", "uriComponents", "Value", Some(0)))?;

	info!(
		"[DocHandler] RPC $tryOpenDocument: URI(external)='{:?}'",
		uri_components.get("external")
	);

	trace!("[DocHandler] $tryOpenDocument full URI components: {:?}", uri_components);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect expects UriComponents, optional languageId, optional content
	let effect = documents_effects::try_open(uri_components.clone(), None, None);

	runtime_state
		.run(effect)
		.await
		.map(|url| json!({ "$mid": 1, "scheme": url.scheme(), "path": url.path(), "external": url.to_string() }))
		.map_err(|e| {
			let op_context = format!("try_open_document for {:?}", uri_components.get("external"));

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, &op_context)
		})
}

pub async fn handle_try_create_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	// Options are optional, clone if present
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
	// try_open effect
	let effect = documents_effects::try_open(Value::Null, language_id_opt, content_opt);

	runtime_state
		.run(effect)
		.await
		.map(|url| json!({ "$mid": 1, "scheme": url.scheme(), "path": url.path(), "external": url.to_string() }))
		.map_err(|e| {
			let op_context = "try_create_document";

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, op_context)
		})
}

pub async fn handle_try_save_document<R:Runtime>(
	app_handle:AppHandle<R>,

	uri_components_val:Value,
) -> Result<Value, String> {
	info!(
		"[DocHandler] RPC $trySaveDocument: URI(external)='{:?}'",
		uri_components_val.get("external")
	);

	trace!(
		"[DocHandler]
$trySaveDocument full URI components: {:?}",
		uri_components_val
	);

	let uri = parse_uri_from_components_param(&uri_components_val, "$trySaveDocument", "uriComponents", Some(0))?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::try_save(uri.clone());

	// Converts bool to Value::Bool
	// CORRECTED: Use closure to call json! macro
	runtime_state.run(effect).await.map(|val| json!(val)).map_err(|e| {
		let op_context = format!("try_save_document for {}", uri);

		error!("[DocHandler] Effect error for {}: {}", op_context, e);

		error_utils::map_common_error_to_rpc_string(e, &op_context)
	})
}

pub async fn handle_try_save_document_as<R:Runtime>(
	app_handle:AppHandle<R>,

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
		Some(0),
	)?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect `try_save_as` with `None` for new_target_uri will trigger UI to
	// pick a new path.
	let effect = documents_effects::try_save_as(original_uri.clone(), None);

	runtime_state
		.run(effect)
		.await
		.map(|new_uri_opt| {
			// Return null if user cancelled save as dialog
			new_uri_opt.map_or(
				Value::Null,
				|new_uri| json!({ "$mid": 1, "scheme": new_uri.scheme(), "path": new_uri.path(), "external": new_uri.to_string() }),
			)
		})
		.map_err(|e| {
			let op_context = format!("try_save_document_as for {}", original_uri);

			error!("[DocHandler] Effect error for {}: {}", op_context, e);

			error_utils::map_common_error_to_rpc_string(e, &op_context)
		})
}

pub async fn handle_save_all<R:Runtime>(app_handle:AppHandle<R>, include_untitled:bool) -> Result<Value, String> {
	info!("[DocHandler] RPC $saveAll: includeUntitled={}", include_untitled);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::save_all(include_untitled);

	// Converts Vec<bool> to Value::Array
	// CORRECTED: Use closure to call json! macro

	runtime_state.run(effect).await.map(|val| json!(val)).map_err(|e| {
		let op_context = "save_all";

		error!("[DocHandler] Effect error for {}: {}", op_context, e);

		error_utils::map_common_error_to_rpc_string(e, op_context)
	})
}

// --- Notification Helpers (Called by Mountain logic/effects) ---

pub async fn notify_model_added<R:Runtime>(app_handle:AppHandle<R>, doc_state:&DocumentState) {
	info!("[DocNotify] Sending $acceptModelAdded for: {}", doc_state.uri);

	trace!("[DocNotify] $acceptModelAdded state: {:?}", doc_state);

	let uri_components = json!({ "$mid": 1, "scheme": doc_state.uri.scheme(), "path": doc_state.uri.path(), "external": doc_state.uri.to_string() });

	// Protocol: $acceptModelAdded(uri: UriComponents, eol: string, versionId:
	// number, lines: string[], languageId: string, isDirty: boolean, encoding:
	// string);

	let payload = json!([
		uri_components,
		doc_state.eol,
		doc_state.version,
		doc_state.lines,
		doc_state.language_id,
		doc_state.is_dirty,
		doc_state.encoding,
	]);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelAdded".to_string(), payload).await {
		error!("[DocNotify] Failed to send $acceptModelAdded for {}: {}", doc_state.uri, e);
	}
}

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

	let uri_components =
		json!({ "$mid": 1, "scheme": doc_uri.scheme(), "path": doc_uri.path(), "external": doc_uri.to_string() });

	let event_data_dto = json!({


	"versionId": doc_version,

	"changes": actual_changes_dto,

	"eol": doc_eol,

	"isUndoing": is_undoing,

	"isRedoing": is_redoing,

	});

	let payload = json!([uri_components, event_data_dto, doc_is_dirty]);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelChanged for {}: {}", doc_uri, e);
	}
}

pub async fn notify_model_saved<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelSaved for: {}", uri);

	let uri_components = json!({ "$mid": 1, "scheme": uri.scheme(), "path": uri.path(), "external": uri.to_string() });

	let payload = json!(uri_components);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelSaved".to_string(), payload).await {
		error!("[DocNotify] Failed to send $acceptModelSaved for {}: {}", uri, e);
	}

	// Note: The responsibility to also send `notify_dirty_state_changed(...,

	// false)`

	// after a successful save is now handled by the
	// `DocumentProvider::save_document`

	// effect implementation in `environment.rs`, which has the full context.
}

pub async fn notify_dirty_state_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, is_dirty:bool) {
	info!("[DocNotify] Sending $acceptDirtyStateChanged ({}) for: {}", is_dirty, uri);

	let uri_components = json!({ "$mid": 1, "scheme": uri.scheme(), "path": uri.path(), "external": uri.to_string() });

	let payload = json!([uri_components, is_dirty]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptDirtyStateChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptDirtyStateChanged for {}: {}", uri, e);
	}
}

pub async fn notify_model_removed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelRemoved for: {}", uri);

	let uri_components = json!({ "$mid": 1, "scheme": uri.scheme(), "path": uri.path(), "external": uri.to_string() });

	let payload = json!(uri_components);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelRemoved".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelRemoved for {}: {}", uri, e);
	}
}

pub async fn notify_language_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, language_id:String) {
	info!(
		"[DocNotify] Sending $acceptModelLanguageChanged ('{}') for: {}",
		language_id, uri
	);

	let uri_components = json!({ "$mid": 1, "scheme": uri.scheme(), "path": uri.path(), "external": uri.to_string() });

	let payload = json!([uri_components, language_id]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptModelLanguageChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelLanguageChanged for {}: {}", uri, e);
	}
}

pub async fn notify_encoding_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, encoding:String) {
	info!("[DocNotify] Sending $acceptEncodingChanged ('{}') for: {}", encoding, uri);

	let uri_components = json!({ "$mid": 1, "scheme": uri.scheme(), "path": uri.path(), "external": uri.to_string() });

	let payload = json!([uri_components, encoding]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptEncodingChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptEncodingChanged for {}: {}", uri, e);
	}
}
