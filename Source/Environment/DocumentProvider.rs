//! # DocumentProvider Implementation
//!
//! Implements the `DocumentProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for all document lifecycle operations, such
//! as opening, saving, and applying text changes, and notifying the `Cocoon`
//! sidecar of these events.

use std::sync::Arc;

use Common::{
	Document::DocumentProvider::DocumentProvider,
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	IPC::IPCProvider::IPCProvider,
};
use async_trait::async_trait;
use log::{error, info, trace, warn};
use serde_json::{Value, json};
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO;

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	/// Opens an existing document from a URI or creates a new untitled
	/// document.
	async fn OpenDocument(
		&self,
		URIComponentsDTO:Value,
		LanguageIdentifier:Option<String>,
		Content:Option<String>,
	) -> Result<Url, CommonError> {
		let URI = Utility::GetURLFromURIComponentsDTO(&URIComponentsDTO)?;
		info!("[DocumentProvider] Opening document: {}", URI);

		let mut OpenDocumentsGuard = self
			.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ExistingDocument) = OpenDocumentsGuard.get(URI.as_str()) {
			info!("[DocumentProvider] Document {} is already open.", URI);
			return Ok(ExistingDocument.URI.clone());
		}

		let FileContent = if let Some(c) = Content {
			c
		} else if URI.scheme() == "file" {
			let FileSystemReader:Arc<dyn FileSystemReader> = self.Require();
			let FilePath = URI.to_file_path().map_err(|_| {
				CommonError::InvalidArgument {
					ArgumentName:"URI".into(),
					Reason:"Cannot convert non-file URI to path".into(),
				}
			})?;
			String::from_utf8(FileSystemReader.ReadFile(&FilePath).await?)
				.map_err(|e| CommonError::FileSystemIO { Path:FilePath, Description:e.to_string() })?
		} else {
			// For non-file schemes without initial content, start with an empty document.
			String::new()
		};

		let NewDocument = DocumentStateDTO::Create(URI.clone(), LanguageIdentifier, FileContent);
		let DTOForNotification = NewDocument.ToDTO();

		OpenDocumentsGuard.insert(URI.to_string(), NewDocument);
		drop(OpenDocumentsGuard);

		NotifyModelAdded(self, &DTOForNotification).await;
		Ok(URI)
	}

	/// Saves the document at the given URI.
	async fn SaveDocument(&self, URI:Url) -> Result<bool, CommonError> {
		info!("[DocumentProvider] Saving document: {}", URI);

		let mut OpenDocumentsGuard = self
			.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Document) = OpenDocumentsGuard.get_mut(URI.as_str()) {
			if URI.scheme() != "file" {
				return Err(CommonError::NotImplemented {
					FeatureName:format!("Saving for URI scheme '{}'", URI.scheme()),
				});
			}

			let FileSystemWriter:Arc<dyn FileSystemWriter> = self.Require();
			let FilePath = URI.to_file_path().unwrap(); // Safe due to scheme check
			let ContentBytes = Document.GetText().into_bytes();

			FileSystemWriter.WriteFile(&FilePath, ContentBytes, true, true).await?;
			Document.IsDirty = false;
			drop(OpenDocumentsGuard);

			NotifyModelSaved(self, &URI).await;
			Ok(true)
		} else {
			Err(CommonError::FileSystemNotFound(URI.to_file_path().unwrap_or_default()))
		}
	}

	/// Saves a document to a new location.
	async fn SaveDocumentAs(&self, _OriginalURI:Url, _NewTargetURI:Option<Url>) -> Result<Option<Url>, CommonError> {
		// A full implementation would use the UserInterfaceProvider to prompt the user.
		warn!("[DocumentProvider] SaveDocumentAs is not fully implemented.");
		Err(CommonError::NotImplemented { FeatureName:"SaveDocumentAs".into() })
	}

	/// Saves all currently dirty documents.
	async fn SaveAllDocuments(&self, _IncludeUntitled:bool) -> Result<Vec<bool>, CommonError> {
		// A full implementation would iterate `ApplicationState.OpenDocuments` and
		// save dirty ones.
		warn!("[DocumentProvider] SaveAllDocuments is not fully implemented.");
		Err(CommonError::NotImplemented { FeatureName:"SaveAllDocuments".into() })
	}

	/// Applies a collection of content changes to a document.
	async fn ApplyDocumentChanges(
		&self,
		URI:Url,
		NewVersionIdentifier:i64,
		ChangesDTOCollection:Value,
		_IsDirtyAfterChange:bool,
		_IsUndoing:bool,
		_IsRedoing:bool,
	) -> Result<(), CommonError> {
		trace!("[DocumentProvider] Applying changes to document: {}", URI);
		let mut OpenDocumentsGuard = self
			.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Document) = OpenDocumentsGuard.get_mut(URI.as_str()) {
			if let Err(e) = Document.ApplyChanges(NewVersionIdentifier, &ChangesDTOCollection) {
				return Err(CommonError::InvalidArgument { ArgumentName:"ChangesDTOCollection".into(), Reason:e });
			}
			Document.IsDirty = true; // Assume any change makes it dirty
		} else {
			warn!("[DocumentProvider] Received changes for unknown document: {}", URI);
			return Ok(());
		}
		drop(OpenDocumentsGuard);

		NotifyModelChanged(self, &URI, NewVersionIdentifier, ChangesDTOCollection).await;
		Ok(())
	}
}

// --- Internal Notification Helpers ---

/// Notifies Cocoon that a new document model has been added.
async fn NotifyModelAdded(Environment:&MountainEnvironment, DocumentStateDTO:&Value) {
	let URIString = DocumentStateDTO.get("URI").and_then(Value::as_str).unwrap_or("unknown");
	info!("[DocumentProvider] Notifying ModelAdded for: {}", URIString);

	let Payload = json!([DocumentStateDTO]);
	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(e) = IPCProvider
		.SendNotificationToSidecar("cocoon-main".to_string(), "$acceptModelAdded".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelAdded for {}: {}", URIString, e);
	}
}

/// Notifies Cocoon that a document's content has changed.
async fn NotifyModelChanged(Environment:&MountainEnvironment, URI:&Url, NewVersion:i64, Changes:Value) {
	info!("[DocumentProvider] Notifying ModelChanged for: {}", URI);

	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });
	let EventData = json!({ "versionId": NewVersion, "changes": Changes });
	let Payload = json!([URIComponents, EventData, true]); // The final `true` is for `isDirty`.
	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(e) = IPCProvider
		.SendNotificationToSidecar("cocoon-main".to_string(), "$acceptModelChanged".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelChanged for {}: {}", URI, e);
	}
}

/// Notifies Cocoon that a document has been saved to disk.
async fn NotifyModelSaved(Environment:&MountainEnvironment, URI:&Url) {
	info!("[DocumentProvider] Notifying ModelSaved for: {}", URI);
	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });
	let Payload = json!([URIComponents]);
	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(e) = IPCProvider
		.SendNotificationToSidecar("cocoon-main".to_string(), "$acceptModelSaved".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelSaved for {}: {}", URI, e);
	}
}

/// Notifies Cocoon that a document has been closed.
pub async fn NotifyModelRemoved(Environment:&MountainEnvironment, URI:&Url) {
	info!("[DocumentProvider] Notifying ModelRemoved for: {}", URI);
	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });
	let Payload = json!([URIComponents]);
	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(e) = IPCProvider
		.SendNotificationToSidecar("cocoon-main".to_string(), "$acceptModelRemoved".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelRemoved for {}: {}", URI, e);
	}
}
