// ---------------------------------------------------------------------------------------------
// Mountain Environment - Document Provider 
// --------------------------------------------------------------------------------------------
// This module implements the `DocumentProvider` trait for
// `MountainEnvironment`. It manages the lifecycle of text documents, including
// opening, saving, and applying content changes. Operations are typically
// delegated to specific handler functions in `handlers::documents`, which
// interact with `AppState` and the filesystem.
// --------------------------------------------------------------------------------------------

use std::path::PathBuf; // Used by some handlers, though not directly in trait sigs here
use std::sync::Arc;

use Land_Common::{
	documents_effects::DocumentProvider, // The trait being implemented
	environment::Requires,
	errors::CommonError,
};
use async_trait::async_trait;
use log::{error, info, trace, warn}; // For logging
use serde_json::Value;
use url::Url;

use crate::{
	app_state, // For app_state::analyze_text_lines_and_eol_for_document_state if used directly
	environment::{
		MountainEnvironment,
		utils::{
			detect_file_encoding_from_bytes,
			detect_language_id_from_file_path,
			map_app_state_lock_error_to_common_error,
			map_io_error_to_common_error,
			// is_path_allowed_for_filesystem_access is called by FsReader/FsWriter impls
		},
	},
	handlers, // For delegating to document handlers
};

// --- DocumentProvider Implementation ---
#[async_trait]
impl DocumentProvider for MountainEnvironment {
	async fn open_document(
		&self,
		uri_components_dto:Value, // DTO from Cocoon or null for new untitled
		language_id_override_opt:Option<String>,
		initial_content_opt:Option<String>,
	) -> Result<Url, CommonError> {
		info!(
			"[Env DocProv] OpenDocument: uri_dto(external)='{:?}', lang_override='{:?}', has_initial_content={}",
			uri_components_dto.get("external").or_else(|| uri_components_dto.get("path")),
			language_id_override_opt,
			initial_content_opt.is_some()
		);

		// Delegate to the specific handler logic.
		// The handler will need access to AppHandle (via MountainEnvironment) and
		// MountainEnvironment itself if it needs to call FsReader methods (e.g.,
		// self.read_file()).
		handlers::documents::handle_open_document_effect_logic(
			self.app_handle.clone(),
			self.clone(), // Pass MountainEnvironment for FsReader access within handler
			uri_components_dto,
			language_id_override_opt,
			initial_content_opt,
		)
		.await
	}

	async fn save_document(&self, uri_to_save:Url) -> Result<bool, CommonError> {
		info!("[Env DocProv] SaveDocument request for URI: {}", uri_to_save);
		// Delegate to handler
		handlers::documents::handle_save_document_effect_logic(
			self.app_handle.clone(),
			self.clone(), // For FsWriter access
			uri_to_save,
		)
		.await
	}

	async fn save_document_as(
		&self,
		original_uri:Url,
		new_uri_target_opt:Option<Url>,
	) -> Result<Option<Url>, CommonError> {
		info!(
			"[Env DocProv] SaveAs: Original='{}', Target (if provided)='{:?}'",
			original_uri, new_uri_target_opt
		);
		// Delegate to handler
		handlers::documents::handle_save_document_as_effect_logic(
			self.app_handle.clone(),
			self.clone(), // For FsWriter & UiProvider access
			original_uri,
			new_uri_target_opt,
		)
		.await
	}

	async fn save_all_documents(&self, include_untitled:bool) -> Result<Vec<bool>, CommonError> {
		info!("[Env DocProv] SaveAll: include_untitled={}", include_untitled);
		// Delegate to handler
		handlers::documents::handle_save_all_documents_effect_logic(
			self.app_handle.clone(),
			self.clone(), // For multiple save_document calls
			include_untitled,
		)
		.await
	}

	async fn apply_document_changes(
		&self,
		uri_to_change:Url,
		new_version_id:i64,
		changes_dto_collection_val:Value, // Array of RpcModelContentChangeDto
		is_dirty_after_change:bool,
		is_undoing_op:bool,
		is_redoing_op:bool,
	) -> Result<(), CommonError> {
		info!(
			"[Env DocProv ApplyChanges] For URI='{}': new_version={}, num_changes={}, is_dirty={}, undo={}, redo={}",
			uri_to_change,
			new_version_id,
			changes_dto_collection_val.as_array().map_or(0, |a| a.len()),
			is_dirty_after_change,
			is_undoing_op,
			is_redoing_op
		);
		// Delegate to handler
		handlers::documents::handle_apply_document_changes_effect_logic(
			self.app_handle.clone(),
			self.clone(), // Not strictly needed by handler if it only modifies AppState
			uri_to_change,
			new_version_id,
			changes_dto_collection_val,
			is_dirty_after_change,
			is_undoing_op,
			is_redoing_op,
		)
		.await
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}
