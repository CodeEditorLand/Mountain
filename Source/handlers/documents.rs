// ---------------------------------------------------------------------------------------------
// Mountain Document Handlers (handlers/documents.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests from Cocoon related to opening, creating, and saving
// documents, primarily by delegating the core logic to the DocumentProvider
// effect system. It also provides helper functions to notify Cocoon (via Vine)
// about document state changes initiated within Mountain (e.g., by effects, UI
// actions).
//
// Responsibilities:
// - Handling `$tryOpenDocument`, `$tryCreateDocument`, `$trySaveDocument`,
//   `$trySaveDocumentAs`, `$saveAll` RPC calls by creating and dispatching
//   corresponding `documents_effects` via the `AppRuntime`.
// - Providing notification helper functions (`notify_model_added`,
//   `notify_model_changed`, `notify_model_saved`, `notify_dirty_state_changed`,
//   `notify_model_removed`, `notify_language_changed`,
//   `notify_encoding_changed`) to be called by Mountain's internal logic
//   (effects, UI handlers) to send `$accept...` notifications to Cocoon via
//   Vine. These notifications carry DTOs compliant with extHost.protocol.ts.
//
// Key Interactions:
// - RPC handlers are called by `track::dispatch_sidecar_request`.
// - RPC handlers use `documents_effects` and `AppRuntime` to perform document
//   operations.
// - Notification helpers use `vine::send_notification_to_sidecar` extensively
//   to keep Cocoon's document shims synchronized.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

// Import effects
use Land_Common::documents_effects;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State};
use url::Url;

use crate::{
	app_state::{AppState, DocumentState},
	runtime::AppRuntime,
	vine,
};

// --- Helper: URI Parsing from Value ---
fn parse_uri_from_components_param(param_val:&Value, method_name:&str) -> Result<Url, String> {
	let uri_str = param_val
		.get("external")
		.and_then(Value::as_str)
		.or_else(|| {
			param_val.get("path").and_then(Value::as_str).map(|p_str| {
				if PathBuf::from(p_str).is_absolute() {
					Url::from_file_path(p_str)
						.map(|u| u.to_string())
						.unwrap_or_else(|_| p_str.to_string())
				} else {
					p_str.to_string()
				}
			})
		})
		.ok_or_else(|| {
			format!(
				"Missing or invalid URI components in {}: 'external' or 'path' string expected.",
				method_name
			)
		})?;

	Url::parse(uri_str).map_err(|e| format!("Failed to parse URI '{}' in {}: {}", uri_str, method_name, e))
}

// --- Helper: lines and EOL from text ---
/// Utility to split text into lines and detect its EOL sequence.
/// This was previously in `app_state.rs` but is a general text utility.
pub fn lines_and_eol_from_text(text:&str) -> (Vec<String>, String) {
	// Default to LF
	let mut detected_eol = "\n";

	if text.contains("\r\n") {
		detected_eol = "\r\n";
	} else if text.contains('\n') {
		detected_eol = "\n";
	} else if text.contains('\r') {
		// For lone \r, standardizing to \n for internal consistency and splitting.
		// VS Code model normalizes EOLs on load.
		detected_eol = "\n";
	}

	// If text only contains \r, splitting by \n results in a single line.
	// If the intent is to split by \r as well, then
	// `text.lines().map(String::from).collect()` might be more robust, but then
	// `detected_eol` logic needs refinement. Sticking with explicit EOL detection
	// for now.
	let lines = text.split(detected_eol).map(String::from).collect();

	(lines, detected_eol.to_string())
}

// --- Handlers for RPC calls from Cocoon ---

pub async fn handle_try_open_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let uri_components = args
		.get(0)
		.ok_or_else(|| "Missing URI components argument for $tryOpenDocument".to_string())?;

	info!(
		"[DocHandler] RPC $tryOpenDocument: URI (external)='{:?}'",
		uri_components.get("external")
	);

	trace!("[DocHandler] $tryOpenDocument full URI components: {:?}", uri_components);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect expects UriComponents, optional languageId, optional content
	let effect = documents_effects::try_open(uri_components.clone(), None, None);

	runtime_state.run(effect).await
		 // Ensure $mid for UriComponents
		.map(|url| json!({ "scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1 }))
		.map_err(|e| {

			error!("[DocHandler] Failed executing try_open effect for {:?}: {}", uri_components.get("external"), e);

			e.to_string()
		})
}

pub async fn handle_try_create_document<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
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
		.map(|url| json!({ "scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1 }))
		.map_err(|e| {
			error!("[DocHandler] Failed executing try_create (via try_open untitled) effect: {}", e);

			e.to_string()
		})
}

pub async fn handle_try_save_document<R:Runtime>(
	app_handle:AppHandle<R>,

	uri_components:Value,
) -> Result<Value, String> {
	info!(
		"[DocHandler] RPC $trySaveDocument: URI (external)='{:?}'",
		uri_components.get("external")
	);

	trace!("[DocHandler] $trySaveDocument full URI components: {:?}", uri_components);

	let uri = parse_uri_from_components_param(&uri_components, "$trySaveDocument")?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::try_save(uri.clone());

	runtime_state.run(effect).await.map(|success| json!(success)).map_err(|e| {
		error!("[DocHandler] Failed executing try_save effect for {}: {}", uri, e);

		e.to_string()
	})
}

pub async fn handle_try_save_document_as<R:Runtime>(
	app_handle:AppHandle<R>,

	uri_components:Value,
) -> Result<Value, String> {
	info!(
		"[DocHandler] RPC $trySaveDocumentAs: Original URI (external)='{:?}'",
		uri_components.get("external")
	);

	trace!(
		"[DocHandler] $trySaveDocumentAs full original URI components: {:?}",
		uri_components
	);

	let original_uri = parse_uri_from_components_param(&uri_components, "$trySaveDocumentAs (original URI)")?;

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	// The effect `try_save_as` with `None` for new_target_uri will trigger UI to
	// pick a new path.
	let effect = documents_effects::try_save_as(original_uri.clone(), None);

	runtime_state
		.run(effect)
		.await
		.map(|new_uri_opt| {
			new_uri_opt.map_or(Value::Null, |new_uri| {
				// Return null if user cancelled save as dialog
				json!({ "scheme": new_uri.scheme(), "path": new_uri.path(), "external": new_uri.to_string(), "$mid": 1 })
			})
		})
		.map_err(|e| {
			error!("[DocHandler] Failed executing try_save_as effect for {}: {}", original_uri, e);

			e.to_string()
		})
}

pub async fn handle_save_all<R:Runtime>(app_handle:AppHandle<R>, include_untitled:bool) -> Result<Value, String> {
	info!("[DocHandler] RPC $saveAll: includeUntitled={}", include_untitled);

	let runtime_state = app_handle.state::<Arc<AppRuntime>>();

	let effect = documents_effects::save_all(include_untitled);

	runtime_state.run(effect).await
		 // save_all effect returns Vec<bool>
		.map(|success_bool_array| json!(success_bool_array))
		.map_err(|e| {

			error!("[DocHandler] Failed executing save_all effect: {}", e);

			e.to_string()
		})
}

// --- Notification Helpers (Called by Mountain logic/effects) ---

pub async fn notify_model_added<R:Runtime>(app_handle:AppHandle<R>, doc_state:&DocumentState) {
	info!("[DocNotify] Sending $acceptModelAdded for: {}", doc_state.uri);

	trace!("[DocNotify] $acceptModelAdded state: {:?}", doc_state);

	let uri_components = json!({

		"scheme": doc_state.uri.scheme(),

		"path": doc_state.uri.path(),

		"external": doc_state.uri.to_string(),

		 // Important for VS Code URI identification
		"$mid": 1
	});

	// Protocol: $acceptModelAdded(uri: UriComponents, eol: string, versionId:
	// number, lines: string[], languageId: string, isDirty: boolean, encoding:
	// string); (Based on Node `ExtHostDocumentsShape.$acceptModelAdded` which has
	// 7 args)
	let payload = json!([
		uri_components,
		doc_state.eol,
		doc_state.version,
		doc_state.lines,
		doc_state.language_id,
		doc_state.is_dirty,
		// Add encoding as per common practice
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

	let uri_components = json!({

		"scheme": doc_uri.scheme(),

		"path": doc_uri.path(),

		"external": doc_uri.to_string(),

		"$mid": 1
	});

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

pub async fn notify_model_saved<R:Runtime>(app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelSaved for: {}", uri);

	let uri_components = json!({

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"$mid": 1
	});

	let payload = json!(uri_components);

	if let Err(e) = vine::send_notification_to_sidecar("cocoon-main", "$acceptModelSaved".to_string(), payload).await {
		error!("[DocNotify] Failed to send $acceptModelSaved for {}: {}", uri, e);
	}

	// The save effect in environment.rs should handle AppState.is_dirty update.
	// This notification only signals the save event to Cocoon.
	// Cocoon's $acceptModelSaved might also imply isDirty=false on its side.
	// If explicit dirty state sync is needed post-save (e.g. if save effect
	// didn't make it clean due to concurrent changes), an additional
	// notify_dirty_state_changed could be sent by the effect. Snippet 1 had:
	// notify_dirty_state_changed(app_handle, uri.clone(), false).await;

	// This is now managed by the save effect in environment.rs which has full
	// context.
}

pub async fn notify_dirty_state_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, is_dirty:bool) {
	info!("[DocNotify] Sending $acceptDirtyStateChanged ({}) for: {}", is_dirty, uri);

	let uri_components = json!({

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"$mid": 1
	});

	let payload = json!([uri_components, is_dirty]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptDirtyStateChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptDirtyStateChanged for {}: {}", uri, e);
	}
}

pub async fn notify_model_removed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url) {
	info!("[DocNotify] Sending $acceptModelRemoved for: {}", uri);

	let uri_components = json!({

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"$mid": 1
	});

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

	let uri_components = json!({

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"$mid": 1
	});

	let payload = json!([uri_components, language_id]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptModelLanguageChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptModelLanguageChanged for {}: {}", uri, e);
	}
}

pub async fn notify_encoding_changed<R:Runtime>(_app_handle:AppHandle<R>, uri:&Url, encoding:String) {
	info!("[DocNotify] Sending $acceptEncodingChanged ('{}') for: {}", encoding, uri);

	let uri_components = json!({

		"scheme": uri.scheme(),

		"path": uri.path(),

		"external": uri.to_string(),

		"$mid": 1
	});

	let payload = json!([uri_components, encoding]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptEncodingChanged".to_string(), payload).await
	{
		error!("[DocNotify] Failed to send $acceptEncodingChanged for {}: {}", uri, e);
	}
}
