// @module DocumentLogic
// @description Contains the core logic for all document lifecycle operations,
// such as opening, saving, and applying text changes.

use std::sync::Arc;

use Common::{
	Environment::Requires,
	error::CommonError,
	fs::{FileSystemReader, FileSystemWriter},
};
use log::{info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::DocumentStateDTO},
	Environment::{MountainEnvironment, Utility},
	Handler::document::NotificationLogic,
};

// Logic for handling the `OpenDocument` effect.
pub async fn OpenDocumentLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	uri_components_DTO:Value,
	language_identifier:Option<String>,
	content:Option<String>,
) -> Result<Url, CommonError> {
	let uri = Utility::GetUrlFromUriDTO(&uri_components_DTO)?;
	info!("[DocumentLogic] Opening document: {}", uri);

	let app_state = app_handle.state::<ApplicationState>();
	let mut open_docs_guard = app_state
		.OpenDocuments
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;

	if let Some(existing_doc) = open_docs_guard.get(uri.as_str()) {
		info!("[DocumentLogic] Document {} is already open.", uri);
		return Ok(existing_doc.Uri.clone());
	}

	let file_content = if let Some(c) = content {
		c
	} else if uri.scheme() == "file" {
		let file_system_reader:Arc<dyn FileSystemReader> = Environment.Require();
		let file_path = uri.to_file_path().map_err(|_| {
			CommonError::InvalidArg { ArgumentName:"Uri".into(), Reason:"Cannot convert non-file URI to path".into() }
		})?;
		String::from_utf8(file_system_reader.ReadFile(&file_path).await?)
			.map_err(|e| CommonError::FileSystemRead { Path:file_path, Description:e.to_string() })?
	} else {
		// For non-file schemes without initial content, start with an empty document.
		String::new()
	};

	let new_doc = DocumentStateDTO::New(uri.clone(), language_identifier, file_content);
	let DTO_for_notification = new_doc.ToDTO();

	open_docs_guard.insert(uri.to_string(), new_doc);
	drop(open_docs_guard);

	NotificationLogic::NotifyModelAdded(app_handle, &DTO_for_notification).await;
	Ok(uri)
}

// Logic for handling the `SaveDocument` effect.
pub async fn SaveDocumentLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	uri:Url,
) -> Result<bool, CommonError> {
	info!("[DocumentLogic] Saving document: {}", uri);
	let app_state = app_handle.state::<ApplicationState>();
	let mut open_docs_guard = app_state
		.OpenDocuments
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;

	if let Some(doc) = open_docs_guard.get_mut(uri.as_str()) {
		if uri.scheme() != "file" {
			return Err(CommonError::NotImplemented {
				FeatureName:format!("Saving for URI scheme '{}'", uri.scheme()),
			});
		}

		let file_system_writer:Arc<dyn FileSystemWriter> = Environment.Require();
		let file_path = uri.to_file_path().unwrap(); // Safe due to scheme check
		let content_bytes = doc.GetText().into_bytes();

		file_system_writer.WriteFile(&file_path, content_bytes, true, true).await?;
		doc.IsDirty = false;
		drop(open_docs_guard);

		NotificationLogic::NotifyModelSaved(app_handle, &uri).await;
		Ok(true)
	} else {
		Err(CommonError::FileSystemNotFound(uri.to_file_path().unwrap_or_default()))
	}
}

// Logic for handling the `ApplyDocumentChanges` effect.
pub async fn ApplyDocumentChangesLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	uri:Url,
	new_version_identifier:i64,
	changes_DTO_collection:Value,
) -> Result<(), CommonError> {
	trace!("[DocumentLogic] Applying changes to document: {}", uri);
	let app_state = app_handle.state::<ApplicationState>();
	let mut open_documents_guard = app_state
		.OpenDocuments
		.lock()
		.map_err(Utility::MapAppStateLockErrorToCommonError)?;

	if let Some(document) = open_documents_guard.get_mut(uri.as_str()) {
		if let Err(e) = document.ApplyChanges(new_version_identifier, &changes_DTO_collection) {
			return Err(CommonError::InvalidArg { ArgumentName:"ChangesDTOCollection".into(), Reason:e });
		}
		document.IsDirty = true; // Assume any change makes it dirty
	} else {
		warn!("[DocumentLogic] Received changes for unknown document: {}", uri);
		return Ok(());
	}
	drop(open_documents_guard);

	NotificationLogic::NotifyModelChanged(app_handle, &uri, new_version_identifier, changes_DTO_collection).await;
	Ok(())
}
