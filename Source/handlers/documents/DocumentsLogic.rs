use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	fs::{FsReader, FsWriter},
};
use log::{info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

/// @module DocumentsLogic
/// @description Contains the core logic for all document lifecycle operations,
/// such as opening, saving, and applying text changes.
use crate::{
	AppState::{AppState::AppState, Dto::DocumentStateDto},
	environment::MountainEnvironment,
	handlers::{self, documents::NotificationLogic, error_utils},
};

/// Logic for handling the `OpenDocument` effect.
pub async fn OpenDocumentLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	UriComponentsDto:Value,
	LanguageIdentifier:Option<String>,
	Content:Option<String>,
) -> Result<Url, CommonError> {
	let Uri = Url::parse(UriComponentsDto.get("external").and_then(Value::as_str).unwrap_or_default())
		.map_err(|_| CommonError::InvalidArg { ArgumentName:"Uri".into(), Reason:"Malformed URI DTO".into() })?;
	info!("[DocumentsLogic] Opening document: {}", Uri);

	let AppStateInstance = AppHandle.state::<AppState>();
	let mut OpenDocsGuard = AppStateInstance.OpenDocuments.lock().unwrap();

	if let Some(ExistingDoc) = OpenDocsGuard.get(Uri.as_str()) {
		return Ok(ExistingDoc.Uri.clone());
	}

	let FsReader:Arc<dyn FsReader> = Environment.Require();
	let FileContent = match Content {
		Some(c) => c,
		None => String::from_utf8(FsReader.ReadFile(&Uri.to_file_path().unwrap()).await?).unwrap_or_default(),
	};

	let NewDoc = DocumentStateDto::New(Uri.clone(), LanguageIdentifier, FileContent);
	let DtoForNotification = NewDoc.ToDto();

	OpenDocsGuard.insert(Uri.to_string(), NewDoc);
	drop(OpenDocsGuard);

	NotificationLogic::NotifyModelAdded(AppHandle, &DtoForNotification).await;
	Ok(Uri)
}

/// Logic for handling the `SaveDocument` effect.
pub async fn SaveDocumentLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Environment:&MountainEnvironment,
	Uri:Url,
) -> Result<bool, CommonError> {
	info!("[DocumentsLogic] Saving document: {}", Uri);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut OpenDocsGuard = AppStateInstance.OpenDocuments.lock().unwrap();

	if let Some(Doc) = OpenDocsGuard.get_mut(Uri.as_str()) {
		let FsWriter:Arc<dyn FsWriter> = Environment.Require();
		let FilePath = Uri.to_file_path().unwrap();
		let ContentBytes = Doc.GetText().into_bytes();

		FsWriter.WriteFile(&FilePath, ContentBytes, true, true).await?;
		Doc.IsDirty = false;
		drop(OpenDocsGuard);

		NotificationLogic::NotifyModelSaved(AppHandle, &Uri).await;
		Ok(true)
	} else {
		Err(CommonError::FsNotFound(Uri.to_file_path().unwrap()))
	}
}

/// Logic for handling the `ApplyDocumentChanges` effect.
pub async fn ApplyDocumentChangesLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Uri:Url,
	NewVersionIdentifier:i64,
	ChangesDtoCollection:Value,
	_IsDirtyAfterChange:bool,
	_IsUndoing:bool,
	_IsRedoing:bool,
) -> Result<(), CommonError> {
	trace!("[DocumentsLogic] Applying changes to document: {}", Uri);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut OpenDocumentsGuard = AppStateInstance.OpenDocuments.lock().unwrap();

	if let Some(Document) = OpenDocumentsGuard.get_mut(Uri.as_str()) {
		if let Err(e) = Document.ApplyChanges(NewVersionIdentifier, &ChangesDtoCollection) {
			return Err(CommonError::InvalidArg { ArgumentName:"ChangesDtoCollection".into(), Reason:e });
		}
		Document.IsDirty = true; // Assume any change makes it dirty
	} else {
		warn!("[DocumentsLogic] Received changes for unknown document: {}", Uri);
		return Ok(());
	}
	drop(OpenDocumentsGuard);

	NotificationLogic::NotifyModelChanged(AppHandle, &Uri, NewVersionIdentifier, ChangesDtoCollection).await;
	Ok(())
}

// ... Full implementations for SaveDocumentAs and SaveAllDocuments would follow
// ...
