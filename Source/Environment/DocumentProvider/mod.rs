//! # DocumentProvider (Environment)
//!
//! Implements the `DocumentProvider` trait, managing the complete lifecycle of
//! document operations including opening, saving, editing, and closing. It
//! maintains document state, coordinates between the frontend (Sky), extension
//! host (Cocoon), and filesystem, and handles both native file URIs and custom
//! scheme URIs.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules for
//! maintainability:
//! - [`OpenDocument`]: Document opening and content resolution (file:// and
//!   custom schemes)
//! - [`SaveOperations`]: SaveDocument, SaveDocumentAs, SaveAllDocuments
//! - [`ApplyChanges`]: ApplyDocumentChanges (incremental text edits)
//! - [`Notifications`]: NotifyModelAdded, NotifyModelChanged, NotifyModelSaved,
//!   NotifyModelRemoved
//!
//! The single `impl DocumentProvider for MountainEnvironment` block in this
//! file delegates to those helper functions. This satisfies Rust's orphan rules
//! while keeping code organized and atomic.

use CommonLibrary::Document::DocumentProvider::DocumentProvider;
use async_trait::async_trait;

// Private helper modules (not re-exported)
mod OpenDocument;
mod SaveOperations;
mod ApplyChanges;
mod Notifications;

#[async_trait]
impl DocumentProvider for crate::Environment::MountainEnvironment::MountainEnvironment {
	async fn OpenDocument(
		&self,
		URIComponentsDTO:serde_json::Value,
		LanguageIdentifier:Option<String>,
		Content:Option<String>,
	) -> Result<url::Url, CommonLibrary::Error::CommonError::CommonError> {
		OpenDocument::open_document(self, URIComponentsDTO, LanguageIdentifier, Content).await
	}

	async fn SaveDocument(&self, URI:url::Url) -> Result<bool, CommonLibrary::Error::CommonError::CommonError> {
		SaveOperations::save_document(self, URI).await
	}

	async fn SaveDocumentAs(
		&self,
		OriginalURI:url::Url,
		NewTargetURI:Option<url::Url>,
	) -> Result<Option<url::Url>, CommonLibrary::Error::CommonError::CommonError> {
		SaveOperations::save_document_as(self, OriginalURI, NewTargetURI).await
	}

	async fn SaveAllDocuments(
		&self,
		IncludeUntitled:bool,
	) -> Result<Vec<bool>, CommonLibrary::Error::CommonError::CommonError> {
		SaveOperations::save_all_documents(self, IncludeUntitled).await
	}

	async fn ApplyDocumentChanges(
		&self,
		URI:url::Url,
		NewVersionIdentifier:i64,
		ChangesDTOCollection:serde_json::Value,
		_IsDirtyAfterChange:bool,
		_IsUndoing:bool,
		_IsRedoing:bool,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ApplyChanges::apply_document_changes(
			self,
			URI,
			NewVersionIdentifier,
			ChangesDTOCollection,
			_IsDirtyAfterChange,
			_IsUndoing,
			_IsRedoing,
		)
		.await
	}
}
