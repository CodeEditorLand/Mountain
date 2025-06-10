use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	fs::{FileSystemReader, FileSystemWriter},
};
use log::{info, trace, warn};
use serde_json::Value;
use tauri::{ApplicationHandle, Manager, RunTime};
use url::Url;

// @module DocumentsLogic
// @description Contains the core logic for all document lifecycle operations,
// such as opening, saving, and applying text changes.
use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::DocumentStateDto},
	Handler::{self, documents::NotificationLogic, error_utils},
	environment::MountainEnvironment,
};

// Logic for handling the `OpenDocument` effect.
pub async fn OpenDocumentLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Environment:&MountainEnvironment,
	UriComponentsDto:Value,
	LanguageIdentifier:Option<String>,
	Content:Option<String>,
) -> Result<Url, CommonError> {
	let Uri = Url::parse(UriComponentsDto.get("external").and_then(Value::as_str).unwrap_or_default())
		.map_err(|_| CommonError::InvalidArg { ArgumentName:"Uri".into(), Reason:"Malformed URI DTO".into() })?;
	info!("[DocumentsLogic] Opening document: {}", Uri);

	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let mut OpenDocsGuard = AppStateInstance.OpenDocuments.lock().unwrap();

	if let Some(ExistingDoc) = OpenDocsGuard.get(Uri.as_str()) {
		return Ok(ExistingDoc.Uri.clone());
	}

	let FileSystemReader:Arc<dyn FileSystemReader> = Environment.Require();
	let FileContent = match Content {
		Some(c) => c,
		None => String::from_utf8(FileSystemReader.ReadFile(&Uri.to_file_path().unwrap()).await?).unwrap_or_default(),
	};

	let NewDoc = DocumentStateDto::New(Uri.clone(), LanguageIdentifier, FileContent);
	let DtoForNotification = NewDoc.ToDto();

	OpenDocsGuard.insert(Uri.to_string(), NewDoc);
	drop(OpenDocsGuard);

	NotificationLogic::NotifyModelAdded(ApplicationHandle, &DtoForNotification).await;
	Ok(Uri)
}

// Logic for handling the `SaveDocument` effect.
pub async fn SaveDocumentLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Environment:&MountainEnvironment,
	Uri:Url,
) -> Result<bool, CommonError> {
	info!("[DocumentsLogic] Saving document: {}", Uri);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let mut OpenDocsGuard = AppStateInstance.OpenDocuments.lock().unwrap();

	if let Some(Doc) = OpenDocsGuard.get_mut(Uri.as_str()) {
		let FileSystemWriter:Arc<dyn FileSystemWriter> = Environment.Require();
		let FilePath = Uri.to_file_path().unwrap();
		let ContentBytes = Doc.GetText().into_bytes();

		FileSystemWriter.WriteFile(&FilePath, ContentBytes, true, true).await?;
		Doc.IsDirty = false;
		drop(OpenDocsGuard);

		NotificationLogic::NotifyModelSaved(ApplicationHandle, &Uri).await;
		Ok(true)
	} else {
		Err(CommonError::FsNotFound(Uri.to_file_path().unwrap()))
	}
}

// Logic for handling the `ApplyDocumentChanges` effect.
pub async fn ApplyDocumentChangesLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Uri:Url,
	NewVersionIdentifier:i64,
	ChangesDtoCollection:Value,
	_IsDirtyAfterChange:bool,
	_IsUndoing:bool,
	_IsRedoing:bool,
) -> Result<(), CommonError> {
	trace!("[DocumentsLogic] Applying changes to document: {}", Uri);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
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

	NotificationLogic::NotifyModelChanged(ApplicationHandle, &Uri, NewVersionIdentifier, ChangesDtoCollection).await;
	Ok(())
}

// ... Full implementations for SaveDocumentAs and SaveAllDocuments would follow
// ...
