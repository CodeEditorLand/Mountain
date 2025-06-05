// ---------------------------------------------------------------------------------------------
// Mountain Environment - Workspace & Workspace Edit Provider
// (environment/workspace_provider.rs)
// --------------------------------------------------------------------------------------------
// This module implements the `WorkspaceProvider` and `WorkspaceEditApplier`
// traits for `MountainEnvironment`.
//
// `WorkspaceProvider` handles querying workspace information (folders, name,
// trust, finding files).
// `WorkspaceEditApplier` handles applying complex `WorkspaceEditDto` operations
// that can modify multiple files or perform file system changes.
//
// Operations are often delegated to handler functions in `handlers::workspace`
// or directly interact with `AppState` and other providers like
// `DocumentProvider` and `FsWriter`.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	document_effects::DocumentProvider, // For applying text edits in WorkspaceEditApplier
	environment::Requires,
	errors::CommonError,
	fs_effects::FsWriter, // For file operations in WorkspaceEditApplier
	language_feature_effects::{
		// DTOs for WorkspaceEdit
		FileEditTypeDto,
		WorkspaceCellEditDto, // Added for completeness, though stubbed
		WorkspaceEditDto,
		WorkspaceFileEditDto,
		WorkspaceTextEditDto,
	},
	workspace_effects::{WorkspaceEditApplier, WorkspaceProvider}, // Traits being implemented
};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use url::Url;

use crate::{
	environment::MountainEnvironment,
	handlers, // For delegating to workspace handlers
};

// --- WorkspaceProvider Implementation ---
#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn get_workspace_folders_info(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		trace!("[Env WkspcProv] GetWorkspaceFoldersInfo");
		handlers::workspace::handle_get_workspace_folders_info_effect_logic(self.app_handle.clone()).await
	}

	async fn get_workspace_folder_info(&self, uri_to_match:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		debug!("[Env WkspcProv] GetWorkspaceFolderInfo for: {}", uri_to_match);
		handlers::workspace::handle_get_workspace_folder_info_effect_logic(self.app_handle.clone(), uri_to_match).await
	}

	async fn get_workspace_name(&self) -> Result<Option<String>, CommonError> {
		debug!("[Env WkspcProv] GetWorkspaceName");
		handlers::workspace::handle_get_workspace_name_effect_logic(self.app_handle.clone()).await
	}

	async fn get_workspace_configuration_path(&self) -> Result<Option<PathBuf>, CommonError> {
		debug!("[Env WkspcProv] GetWorkspaceConfigurationPath");
		handlers::workspace::handle_get_workspace_configuration_path_effect_logic(self.app_handle.clone()).await
	}

	async fn is_workspace_trusted(&self) -> Result<bool, CommonError> {
		debug!("[Env WkspcProv] IsWorkspaceTrusted");
		handlers::workspace::handle_is_workspace_trusted_effect_logic(self.app_handle.clone()).await
	}

	async fn request_workspace_trust(&self, options:Option<Value>) -> Result<bool, CommonError> {
		info!("[Env WkspcProv] RequestWorkspaceTrust, options: {:?}", options);
		// This handler would typically use UiProvider to show a dialog.
		handlers::workspace::handle_request_workspace_trust_effect_logic(self.app_handle.clone(), self.clone(), options)
			.await
	}

	async fn find_files_in_workspace(
		&self,
		include_pattern_dto:Value,
		exclude_pattern_dto:Option<Value>,
		max_results:Option<usize>,
		use_ignore_files:bool,
		follow_symlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!(
			"[Env WkspcProv] FindFilesInWorkspace: include='{:?}', exclude='{:?}'",
			include_pattern_dto, exclude_pattern_dto
		);
		handlers::workspace::handle_find_files_in_workspace_effect_logic(
			self.app_handle.clone(),
			include_pattern_dto,
			exclude_pattern_dto,
			max_results,
			use_ignore_files,
			follow_symlinks,
		)
		.await
	}

	async fn open_file(&self, path:PathBuf) -> Result<(), CommonError> {
		info!("[Env WkspcProv] OpenFile (workspace context): {}", path.display());
		// This might be for opening files in a way that doesn't involve the full
		// DocumentProvider lifecycle, or it's a convenience that resolves paths
		// relative to the workspace before handing off to DocumentProvider.
		// For now, delegate to a handler.
		handlers::workspace::handle_open_file_effect_logic(self.app_handle.clone(), self.clone(), path).await
	}
}

// --- WorkspaceEditApplier Implementation ---
#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	async fn apply_workspace_edit(&self, edit_dto:WorkspaceEditDto) -> Result<bool, CommonError> {
		info!(
			"[Env WkspcEditApplier] Applying WorkspaceEdit with {} edits. Top-level metadata: {:?}",
			edit_dto.edits.len(),
			edit_dto.metadata.as_ref().and_then(|m| m.get("label"))
		);

		// Acquire necessary provider capabilities from self.
		let doc_provider:Arc<dyn DocumentProvider + Send + Sync> = self.require();
		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.require();

		// TODO: Implement sophisticated edit merging and ordering if necessary.
		// For now, apply sequentially.
		for (index, edit_entry_val) in edit_dto.edits.iter().enumerate() {
			trace!(
				"[Env WkspcEditApplier] Processing edit entry #{}: Type {:?}",
				index,
				edit_entry_val.get("_type").and_then(Value::as_u64)
			);

			let edit_type_num = edit_entry_val.get("_type").and_then(Value::as_u64);
			// Deserialize FileEditTypeDto from the number
			let edit_type_dto_opt:Option<FileEditTypeDto> =
				edit_type_num.and_then(|v_u64| serde_json::from_value(Value::from(v_u64)).ok());

			match edit_type_dto_opt {
				Some(FileEditTypeDto::Text) | Some(FileEditTypeDto::Snippet) => {
					let text_op =
						serde_json::from_value::<WorkspaceTextEditDto>(edit_entry_val.clone()).map_err(|e| {
							CommonError::InvalidArg(
								"text_edit_entry".to_string(),
								format!("Deserialize WorkspaceTextEditDto failed: {}", e),
							)
						})?;

					let target_uri = handlers::documents::parse_uri_from_components_param(
						&text_op.resource,
						"apply_edit_text",
						"resource",
						None,
					)
					.map_err(|e_str| CommonError::InvalidArg("resource_uri".to_string(), e_str))?;

					// Adapt WorkspaceTextEditDto.edit (ISingleEditOperation) to
					// RpcModelContentChangeDto
					let single_edit_op_val = text_op.edit; // This is Value
					let rpc_model_content_change = json!({
						"range": single_edit_op_val.get("range").cloned().unwrap_or(Value::Null),
						"text": single_edit_op_val.get("text").cloned().unwrap_or(Value::Null),
						"eol": single_edit_op_val.get("eol").cloned(), // Optional
						// If FileEditTypeDto::Snippet, DocumentProvider.apply_document_changes might handle this.
					});
					let changes_array_val = Value::Array(vec![rpc_model_content_change]);
					let version_id_for_apply = text_op.version_id.map(|v| v as i64).unwrap_or(-1); // -1 for "next version"

					info!("[Env WkspcEditApplier] Applying TextEdit to: {}", target_uri);
					doc_provider
						.apply_document_changes(
							target_uri.clone(),
							version_id_for_apply,
							changes_array_val,
							true,
							false,
							false, // Assume dirty, not undo/redo
						)
						.await?;
				},
				Some(FileEditTypeDto::File) => {
					let file_op =
						serde_json::from_value::<WorkspaceFileEditDto>(edit_entry_val.clone()).map_err(|e| {
							CommonError::InvalidArg(
								"file_edit_entry".to_string(),
								format!("Deserialize WorkspaceFileEditDto failed: {}", e),
							)
						})?;

					let old_url_opt = file_op.old_uri.as_ref().and_then(|v| {
						handlers::documents::parse_uri_from_components_param(v, "apply_edit_file_old", "old_uri", None)
							.ok()
					});
					let new_url_opt = file_op.new_uri.as_ref().and_then(|v| {
						handlers::documents::parse_uri_from_components_param(v, "apply_edit_file_new", "new_uri", None)
							.ok()
					});

					let overwrite = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("overwrite"))
						.and_then(Value::as_bool)
						.unwrap_or(false);
					let recursive = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("recursive"))
						.and_then(Value::as_bool)
						.unwrap_or(false); // For delete
					let ignore_if_not_exists = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("ignoreIfNotExists"))
						.and_then(Value::as_bool)
						.unwrap_or(false); // For delete

					if let (Some(old_uri), Some(new_uri)) = (old_url_opt.as_ref(), new_url_opt.as_ref()) {
						// RENAME
						info!("[Env WkspcEditApplier] Applying File Rename: {} -> {}", old_uri, new_uri);
						if old_uri.scheme() != "file" || new_uri.scheme() != "file" {
							return Err(CommonError::NotImplemented("Rename for non-file URIs".into()));
						}
						fs_writer
							.rename(&PathBuf::from(old_uri.path()), &PathBuf::from(new_uri.path()), overwrite)
							.await?;
					} else if let Some(new_uri) = new_url_opt.as_ref() {
						// CREATE
						info!("[Env WkspcEditApplier] Applying File Create: {}", new_uri);
						if new_uri.scheme() != "file" {
							return Err(CommonError::NotImplemented("Create for non-file URIs".into()));
						}
						// Assuming create of an empty file. If `options.contents` were present, use
						// that.
						fs_writer
							.write_file(&PathBuf::from(new_uri.path()), Vec::new(), true /* create */, overwrite)
							.await?;
					} else if let Some(old_uri) = old_url_opt.as_ref() {
						// DELETE
						info!("[Env WkspcEditApplier] Applying File Delete: {}", old_uri);
						if old_uri.scheme() != "file" {
							return Err(CommonError::NotImplemented("Delete for non-file URIs".into()));
						}
						let path_to_delete = PathBuf::from(old_uri.path());
						if ignore_if_not_exists && !tokio::fs::try_exists(&path_to_delete).await.unwrap_or(false) {
							debug!(
								"[Env WkspcEditApplier] Delete skipped for non-existent {} (ignoreIfNotExists=true)",
								old_uri
							);
						} else {
							fs_writer
								.delete(&path_to_delete, recursive, false /* useTrash default to false */)
								.await?;
						}
					} else {
						return Err(CommonError::InvalidArg(
							"file_edit_entry".to_string(),
							"File operation DTO missing oldUri and newUri.".to_string(),
						));
					}
				},
				Some(FileEditTypeDto::Cell)
				| Some(FileEditTypeDto::CellReplace)
				| Some(FileEditTypeDto::CellMetadata)
				| Some(FileEditTypeDto::DocumentMetadata) => {
					// Attempt to deserialize as WorkspaceCellEditDto to log more info if needed
					let cell_op_detail = serde_json::from_value::<WorkspaceCellEditDto>(edit_entry_val.clone());
					warn!(
						"[Env WkspcEditApplier] Notebook cell edit application is STUBBED for type: {:?}. Edit detail \
						 (if parsed): {:?}",
						edit_type_dto_opt,
						cell_op_detail.ok()
					);
					// TODO: Implement cell edit application using a
					// NotebookDocumentProvider or similar.
				},
				None => {
					// _type field was missing or not a known u64/u32 value
					warn!(
						"[Env WkspcEditApplier] Unknown or missing _type in WorkspaceEditDto entry: {:?}. Edit: {:?}",
						edit_type_num, edit_entry_val
					);
					return Err(CommonError::InvalidArg(
						"edit_entry._type".to_string(),
						"Unknown or missing edit _type".to_string(),
					));
				},
			}
		}
		Ok(true) // Assume success if all operations passed or were non-critical stubs.
	}
}

// --- Requires Implementations ---
impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}
