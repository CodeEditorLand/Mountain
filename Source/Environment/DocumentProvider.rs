// File: Mountain/Source/Environment/DocumentProvider.rs
//
// # Architectural Role: Document Lifecycle and State Management
//
// DocumentProvider implements the DocumentProvider trait, managing the complete
// lifecycle of document operations including opening, saving, editing, and
// closing. It maintains document state, coordinates between the frontend (Sky),
// extension host (Cocoon), and filesystem, and handles both native file URIs
// and custom scheme URIs.
//
// # Responsibilities
//
// 1. **Document State Management**: Maintains all open documents in
//    ApplicationState, tracking content, version, dirty status, and metadata.
//
// 2. **Document Persistence**: Handles saving documents to the filesystem,
//    including Save and Save As operations.
//
// 3. **Change Tracking**: Applies incremental text edits to documents, tracking
//    version identifiers for collaboration and undo/redo.
//
// 4. **URI Scheme Support**: Supports both native file:// URIs and custom
//    scheme URIs (e.g., untitled:, git:, vscode-vfs:) via
//    TextDocumentContentProvider.
//
// 5. **Event Orchestration**: Emits events to Sky and Cocoon for document
//    lifecycle changes (Opened, Saved, Changed, Closed, Renamed).
//
// 6. **Bulk Operations**: Supports Save All to handle multiple dirty documents
//    efficiently.
//
// # Document State Model
//
// Each open document is represented in ApplicationState.OpenDocuments with:
// - URI: Uniform Resource Identifier (file:// or custom scheme)
// - LanguageIdentifier: Language ID (e.g., rust, typescript, markdown)
// - TextContent: Full document text
// - Version: Monotonically increasing version number (for LSP synchronization)
// - LineCount: Cached line count for performance
// - ChangeCount: Number of incremental changes applied
// - IsDirty: Whether document has unsaved changes
//
// # Document Open Flow
//
// 1. Frontend requests to open a URI with optional content
// 2. Mountain checks if document is already open - if yes, refocus existing
//    model
// 3. If file:// URI: read content from filesystem via FileSystemReader
// 4. If custom URI: request content from extension's
//    TextDocumentContentProvider via IPC
// 5. Create DocumentStateDTO and store in ApplicationState.OpenDocuments
// 6. Detect language from file extension or explicit parameter
// 7. Emit "sky://documents/open" event to frontend
// 8. Send $acceptModelAdded notification to Cocoon
//
// # Document Edit Flow
//
// 1. Frontend applies text edits with new version number
// 2. Mountain applies changes to DocumentStateDTO text content
// 3. Update version, line count, change count, and dirty status
// 4. Send $acceptModelChanged notification to Cocoon
// 5. Cocoon forwards to language servers for analysis
//
// # Document Save Flow
//
// 1. Frontend requests to save a URI
// 2. Mountain retrieves document text content
// 3. Write to filesystem via FileSystemWriter
// 4. Clear dirty flag
// 5. Emit "sky://documents/saved" event to frontend
// 6. Send $acceptModelSaved notification to Cocoon
// 7. Cocoon notifies language servers to update diagnostics
//
// # Patterns Borrowed from VSCode
//
// - **Text Model Service**: Inspired by VSCode's ITextModelService for managing
//   text document instances with lifecycle and change events.
//
// - **TextDocumentContentProvider**: Like VSCode's pattern for custom URI
//   schemes, allows extensions to provide document content dynamically.
//
// - **Synchronized Versions**: Mimics VSCode's versioning for seamless LSP
//   synchronization with Language Servers.
//
// - **Change Events**: Emits granular change notifications like VSCode's
//   onDidChangeTextDocument event.
//
// # TODOs
//
// - [ ] Implement document revert to last saved
// - [ ] Add document backup before save for crash recovery
// - [ ] Implement proper encoding detection and conversion (UTF-8, UTF-16,
//   etc.)
// - [ ] Add document state persistence across application restarts
// - [ ] Implement document close cleanup and resource releasing
// - [ ] Add document line ending normalization (LF, CRLF)
// - [ ] Implement document diff and merge support
// - [ ] Add support for large file handling (streaming, memory limits)
// - [ ] Implement document auto-save with user-configurable delay
// - [ ] TODO (Mountain→Air Split): Consider delegating bulk save operations to
//   Air for improved performance with large workspaces. Air could handle save
//   batches in the background and report progress via gRPC status events.

use std::{path::PathBuf, sync::Arc};

use uuid::Uuid;
use CommonLibrary::{
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

			match ExistingDocument.ToDTO() {
				Ok(DTO) => {
					if let Err(Error) = self.ApplicationHandle.emit("sky://documents/open", DTO) {
						error!("[DocumentProvider] Failed to emit document open event: {}", Error);
					}
				},

				Err(Error) => {
					error!("[DocumentProvider] Failed to serialize existing document DTO: {}", Error);
				},
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
		let NewDocument = DocumentStateDTO::Create(URI.clone(), LanguageIdentifier, FileContent)?;

		let DTOForNotification = NewDocument.ToDTO()?;

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
				// For non-file URIs, use temporary file location
				if URI.scheme() != "file" {
					info!("[DocumentProvider] Saving non-file URI '{}' to temporary location", URI);
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
				DocumentStateDTO::Create(NewURI.clone(), OldDocument.map(|d| d.LanguageIdentifier), OriginalContent)?;

			let DTO = NewDocument.ToDTO()?;

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
	async fn SaveAllDocuments(&self, IncludeUntitled:bool) -> Result<Vec<bool>, CommonError> {
		info!(
			"[DocumentProvider] SaveAllDocuments called (IncludeUntitled: {})",
			IncludeUntitled
		);

		let URIsToSave:Vec<Url> = {
			let OpenDocumentsGuard = self
				.ApplicationState
				.OpenDocuments
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			OpenDocumentsGuard
				.values()
				.filter(|Document| {
					// Include documents that are dirty
					if !Document.IsDirty {
						return false;
					}

					// Include only file-scheme documents unless IncludeUntitled is true
					if !IncludeUntitled && Document.URI.scheme() != "file" {
						return false;
					}

					true
				})
				.map(|Document| Document.URI.clone())
				.collect()
		};

		let mut Results = Vec::with_capacity(URIsToSave.len());

		info!("[DocumentProvider] Saving {} dirty document(s)", URIsToSave.len());

		for URI in URIsToSave {
			let Result = self.SaveDocument(URI.clone()).await;

			match &Result {
				Ok(_) => {
					info!("[DocumentProvider] Successfully saved {}", URI);
				},
				Err(Error) => {
					error!("[DocumentProvider] Failed to save {}: {}", URI, Error);
				},
			}

			Results.push(Result.is_ok());
		}

		Ok(Results)
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
				Document.ApplyChanges(NewVersionIdentifier, &ChangesDTOCollection)?;
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
