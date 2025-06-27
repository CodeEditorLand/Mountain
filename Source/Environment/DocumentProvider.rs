// File: Mountain/Source/Environment/DocumentProvider.rs
// Role: Implements the `DocumentProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Core logic for all document lifecycle operations (open, save, change).
//   - Notifies `Cocoon` (extension host) and `Sky` (frontend) of these events.
//   - Handles content resolution for both native (`file`) and custom URI
//     schemes.

//! # DocumentProvider Implementation
//!
//! Implements the `DocumentProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for all document lifecycle operations, such
//! as opening, saving, and applying text changes, and notifying the `Cocoon`
//! sidecar and `Sky` frontend of these events.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	Document::DocumentProvider::DocumentProvider,
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{ReadFile::ReadFile, WriteFileBytes::WriteFileBytes},
	IPC::IPCProvider::IPCProvider,
	UserInterface::{DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO, ShowSaveDialog::ShowSaveDialog},
};
use async_trait::async_trait;
use log::{error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{Emitter, Manager};
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{
	ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	/// Opens a document. If the URI scheme is not native (`file`), it attempts
	/// to resolve the content from a registered sidecar provider
	/// (`TextDocumentContentProvider`).
	async fn OpenDocument(
		&self,

		URIComponentsDTO:Value,

		LanguageIdentifier:Option<String>,

		Content:Option<String>,
	) -> Result<Url, CommonError> {
		let URI = Utility::GetURLFromURIComponentsDTO(&URIComponentsDTO)?;

		info!("[DocumentProvider] Opening document: {}", URI);

		// First, check if the document is already open.
		if let Some(ExistingDocument) = self
			.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(URI.as_str())
		{
			info!("[DocumentProvider] Document {} is already open.", URI);

			let DTO = ExistingDocument.ToDTO();

			if let Err(Error) = self.ApplicationHandle.emit("sky://documents/open", DTO) {
				error!("[DocumentProvider] Failed to emit document open event: {}", Error);
			}

			return Ok(ExistingDocument.URI.clone());
		}

		// Resolve the content based on the URI scheme.
		let FileContent = if let Some(c) = Content {
			c
		} else if URI.scheme() == "file" {
			let FilePath = URI.to_file_path().map_err(|_| {
				CommonError::InvalidArgument {
					ArgumentName:"URI".into(),

					Reason:"Cannot convert non-file URI to path".into(),
				}
			})?;

			let RunTime = self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

			let FileContentBytes = RunTime.Run(ReadFile(FilePath.clone())).await?;

			String::from_utf8(FileContentBytes)
				.map_err(|Error| CommonError::FileSystemIO { Path:FilePath, Description:Error.to_string() })?
		} else {
			// Custom scheme: attempt to resolve from a sidecar provider.
			info!(
				"[DocumentProvider] Non-native scheme '{}'. Attempting to resolve from sidecar.",
				URI.scheme()
			);

			let IPCProvider:Arc<dyn IPCProvider> = self.Require();

			let RpcResult = IPCProvider
				.SendRequestToSideCar(
					// In a multi-host world, we'd look this up
					"cocoon-main".to_string(),
					"$provideTextDocumentContent".to_string(),
					json!([URIComponentsDTO]),
					10000,
				)
				.await?;

			RpcResult.as_str().map(String::from).ok_or_else(|| {
				CommonError::IPCError {
					Description:format!("Failed to get valid string content for custom URI scheme '{}'", URI.scheme()),
				}
			})?
		};

		// The rest of the flow is the same for all schemes.
		let NewDocument = DocumentStateDTO::Create(URI.clone(), LanguageIdentifier, FileContent);

		let DTOForNotification = NewDocument.ToDTO();

		self.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(URI.to_string(), NewDocument);

		if let Err(Error) = self.ApplicationHandle.emit("sky://documents/open", DTOForNotification.clone()) {
			error!("[DocumentProvider] Failed to emit document open event: {}", Error);
		}

		NotifyModelAdded(self, &DTOForNotification).await;

		Ok(URI)
	}

	/// Saves the document at the given URI.
	async fn SaveDocument(&self, URI:Url) -> Result<bool, CommonError> {
		info!("[DocumentProvider] Saving document: {}", URI);

		let (ContentBytes, FilePath) = {
			let mut OpenDocumentsGuard = self
				.ApplicationState
				.OpenDocuments
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(Document) = OpenDocumentsGuard.get_mut(URI.as_str()) {
				if URI.scheme() != "file" {
					return Err(CommonError::NotImplemented {
						FeatureName:format!(
							"Saving for URI scheme '{}' is not supported via this method.",
							URI.scheme()
						),
					});
				}

				Document.IsDirty = false;

				(
					Document.GetText().into_bytes(),
					URI.to_file_path().map_err(|_| {
						CommonError::InvalidArgument {
							ArgumentName:"URI".into(),

							Reason:"Cannot convert file URI to path".into(),
						}
					})?,
				)
			} else {
				return Err(CommonError::FileSystemNotFound(URI.to_file_path().unwrap_or_default()));
			}
		};

		let RunTime = self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		RunTime.Run(WriteFileBytes(FilePath, ContentBytes, true, true)).await?;

		if let Err(Error) = self
			.ApplicationHandle
			.emit("sky://documents/saved", json!({ "uri": URI.to_string() }))
		{
			error!("[DocumentProvider] Failed to emit document saved event: {}", Error);
		}

		NotifyModelSaved(self, &URI).await;

		Ok(true)
	}

	/// Saves a document to a new location.
	async fn SaveDocumentAs(&self, OriginalURI:Url, NewTargetURI:Option<Url>) -> Result<Option<Url>, CommonError> {
		info!("[DocumentProvider] Saving document as: {}", OriginalURI);

		let RunTime = self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		let NewFilePath = match NewTargetURI {
			Some(uri) => uri.to_file_path().ok(),

			None => RunTime.Run(ShowSaveDialog(Some(SaveDialogOptionsDTO::default()))).await?,
		};

		let Some(NewPath) = NewFilePath else { return Ok(None) };

		let NewURI = Url::from_file_path(&NewPath).map_err(|_| {
			CommonError::InvalidArgument {
				ArgumentName:"NewPath".into(),

				Reason:"Could not convert new path to URI".into(),
			}
		})?;

		let OriginalContent = {
			let Guard = self
				.ApplicationState
				.OpenDocuments
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			Guard
				.get(OriginalURI.as_str())
				.map(|doc| doc.GetText())
				.ok_or_else(|| CommonError::FileSystemNotFound(PathBuf::from(OriginalURI.path())))?
		};

		RunTime
			.Run(WriteFileBytes(NewPath, OriginalContent.clone().into_bytes(), true, true))
			.await?;

		let NewDocumentState = {
			let mut Guard = self
				.ApplicationState
				.OpenDocuments
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			let OldDocument = Guard.remove(OriginalURI.as_str());

			let NewDocument =
				DocumentStateDTO::Create(NewURI.clone(), OldDocument.map(|d| d.LanguageIdentifier), OriginalContent);

			let DTO = NewDocument.ToDTO();

			Guard.insert(NewURI.to_string(), NewDocument);

			DTO
		};

		NotifyModelRemoved(self, &OriginalURI).await;

		NotifyModelAdded(self, &NewDocumentState).await;

		if let Err(Error) = self.ApplicationHandle.emit(
			"sky://documents/renamed",
			json!({ "oldUri": OriginalURI.to_string(), "newUri": NewURI.to_string() }),
		) {
			error!("[DocumentProvider] Failed to emit document renamed event: {}", Error);
		}

		Ok(Some(NewURI))
	}

	/// Saves all currently dirty documents.
	async fn SaveAllDocuments(&self, _IncludeUntitled:bool) -> Result<Vec<bool>, CommonError> {
		warn!("[DocumentProvider] SaveAllDocuments is not implemented.");

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

		{
			let mut OpenDocumentsGuard = self
				.ApplicationState
				.OpenDocuments
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(Document) = OpenDocumentsGuard.get_mut(URI.as_str()) {
				Document.ApplyChanges(NewVersionIdentifier, &ChangesDTOCollection);
			} else {
				warn!("[DocumentProvider] Received changes for unknown document: {}", URI);

				return Ok(());
			}
		}

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

	if let Err(Error) = IPCProvider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelAdded".to_string(), Payload)
		.await
	{
		error!(
			"[DocumentProvider] Failed to send $acceptModelAdded for {}: {}",
			URIString, Error
		);
	}
}

/// Notifies Cocoon that a document's content has changed.
async fn NotifyModelChanged(Environment:&MountainEnvironment, URI:&Url, NewVersion:i64, Changes:Value) {
	info!("[DocumentProvider] Notifying ModelChanged for: {}", URI);

	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });

	let EventData = json!({ "versionId": NewVersion, "changes": Changes, "isDirty": true });

	let Payload = json!([URIComponents, EventData]);

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(Error) = IPCProvider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelChanged".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelChanged for {}: {}", URI, Error);
	}
}

/// Notifies Cocoon that a document has been saved to disk.
async fn NotifyModelSaved(Environment:&MountainEnvironment, URI:&Url) {
	info!("[DocumentProvider] Notifying ModelSaved for: {}", URI);

	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });

	let Payload = json!([URIComponents]);

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(Error) = IPCProvider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelSaved".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelSaved for {}: {}", URI, Error);
	}
}

/// Notifies Cocoon that a document has been closed or renamed.
pub async fn NotifyModelRemoved(Environment:&MountainEnvironment, URI:&Url) {
	info!("[DocumentProvider] Notifying ModelRemoved for: {}", URI);

	let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });

	let Payload = json!([URIComponents]);

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	if let Err(Error) = IPCProvider
		.SendNotificationToSideCar("cocoon-main".to_string(), "$acceptModelRemoved".to_string(), Payload)
		.await
	{
		error!("[DocumentProvider] Failed to send $acceptModelRemoved for {}: {}", URI, Error);
	}
}
