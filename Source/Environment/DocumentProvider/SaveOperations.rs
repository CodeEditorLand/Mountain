//! Document save operations.
//!
//! Handles SaveDocument, SaveDocumentAs, and SaveAllDocuments.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::WriteFileBytes::WriteFileBytes,
	IPC::SkyEvent::SkyEvent,
	UserInterface::{DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO, ShowSaveDialog::ShowSaveDialog},
};
use serde_json::json;
use tauri::{Emitter, Manager};
use url::Url;

use crate::{
	ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO,
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Saves the document at the given URI.
pub(super) async fn save_document(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	uri:Url,
) -> Result<bool, CommonError> {
	dev_log!("model", "[DocumentProvider] Saving document: {}", uri);

	let (content_bytes, file_path) = {
		let mut open_documents_guard = environment
			.ApplicationState
			.Feature
			.Documents
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(document) = open_documents_guard.get_mut(uri.as_str()) {
			// For non-file URIs, use temporary file location
			if uri.scheme() != "file" {
				dev_log!(
					"model",
					"[DocumentProvider] Saving non-file URI '{}' to temporary location",
					uri
				);
			}

			document.IsDirty = false;

			(
				document.GetText().into_bytes(),
				uri.to_file_path().map_err(|_| {
					CommonError::InvalidArgument {
						ArgumentName:"URI".into(),
						Reason:"Cannot convert file URI to path".into(),
					}
				})?,
			)
		} else {
			return Err(CommonError::FileSystemNotFound(uri.to_file_path().unwrap_or_default()));
		}
	};

	let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	runtime.Run(WriteFileBytes(file_path, content_bytes, true, true)).await?;

	if let Err(error) = environment
		.ApplicationHandle
		.emit(SkyEvent::DocumentsSaved.AsStr(), json!({ "uri": uri.to_string() }))
	{
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to emit document saved event: {}",
			error
		);
	}

	crate::Environment::DocumentProvider::Notifications::notify_model_saved(environment, &uri).await;

	Ok(true)
}

/// Saves a document to a new location.
pub(super) async fn save_document_as(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	original_uri:Url,
	new_target_uri:Option<Url>,
) -> Result<Option<Url>, CommonError> {
	dev_log!("model", "[DocumentProvider] Saving document as: {}", original_uri);

	let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let new_file_path = match new_target_uri {
		Some(uri) => uri.to_file_path().ok(),
		None => runtime.Run(ShowSaveDialog(Some(SaveDialogOptionsDTO::default()))).await?,
	};

	let Some(new_path) = new_file_path else { return Ok(None) };

	let new_uri = Url::from_file_path(&new_path).map_err(|_| {
		CommonError::InvalidArgument {
			ArgumentName:"NewPath".into(),
			Reason:"Could not convert new path to URI".into(),
		}
	})?;

	let original_content = {
		let guard = environment
			.ApplicationState
			.Feature
			.Documents
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		guard
			.get(original_uri.as_str())
			.map(|doc| doc.GetText())
			.ok_or_else(|| CommonError::FileSystemNotFound(PathBuf::from(original_uri.path())))?
	};

	runtime
		.Run(WriteFileBytes(new_path, original_content.clone().into_bytes(), true, true))
		.await?;

	let new_document_state = {
		let mut guard = environment
			.ApplicationState
			.Feature
			.Documents
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let old_document = guard.remove(original_uri.as_str());

		let new_document =
			DocumentStateDTO::Create(new_uri.clone(), old_document.map(|d| d.LanguageIdentifier), original_content)?;

		let dto = new_document.ToDTO()?;

		guard.insert(new_uri.to_string(), new_document);

		dto
	};

	crate::Environment::DocumentProvider::Notifications::notify_model_removed(environment, &original_uri).await;

	crate::Environment::DocumentProvider::Notifications::notify_model_added(environment, &new_document_state).await;

	if let Err(error) = environment.ApplicationHandle.emit(
		SkyEvent::DocumentsRenamed.AsStr(),
		json!({ "oldUri": original_uri.to_string(), "newUri": new_uri.to_string() }),
	) {
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to emit document renamed event: {}",
			error
		);
	}

	Ok(Some(new_uri))
}

/// Saves all currently dirty documents.
pub(super) async fn save_all_documents(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	include_untitled:bool,
) -> Result<Vec<bool>, CommonError> {
	dev_log!(
		"model",
		"[DocumentProvider] SaveAllDocuments called (IncludeUntitled: {})",
		include_untitled
	);

	let uris_to_save:Vec<Url> = {
		let open_documents_guard = environment
			.ApplicationState
			.Feature
			.Documents
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		open_documents_guard
			.values()
			.filter(|document| {
				// Include documents that are dirty
				if !document.IsDirty {
					return false;
				}

				// Include only file-scheme documents unless IncludeUntitled is true
				if !include_untitled && document.URI.scheme() != "file" {
					return false;
				}

				true
			})
			.map(|document| document.URI.clone())
			.collect()
	};

	let mut results = Vec::with_capacity(uris_to_save.len());

	dev_log!("model", "[DocumentProvider] Saving {} dirty document(s)", uris_to_save.len());

	for uri in uris_to_save {
		let result = save_document(environment, uri.clone()).await;

		match &result {
			Ok(_) => {
				dev_log!("model", "[DocumentProvider] Successfully saved {}", uri);
			},
			Err(error) => {
				dev_log!("model", "error: [DocumentProvider] Failed to save {}: {}", uri, error);
			},
		}

		results.push(result.is_ok());
	}

	Ok(results)
}
