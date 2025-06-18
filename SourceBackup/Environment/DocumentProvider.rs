// @module DocumentProvider (Environment)
// @description Implements the `DocumentsProvider` trait for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{document::DocumentsProvider, Environment::Requires, error::CommonError};
use serde_json::Value;
use url::Url;

use super::MountainEnvironment;
use crate::Handler::document as DocumentHandler;

#[async_trait]
impl DocumentsProvider for MountainEnvironment {
	// Opens an existing document from a URI or creates a new untitled
	// document.
	async fn OpenDocument(
		&self,
		uri_components_DTO:Value,
		language_identifier:Option<String>,
		content:Option<String>,
	) -> Result<Url, CommonError> {
		DocumentHandler::OpenDocumentLogic(
			&self.ApplicationHandle,
			self,
			uri_components_DTO,
			language_identifier,
			content,
		)
		.await
	}

	// Saves the document at the given URI.
	async fn SaveDocument(&self, uri:Url) -> Result<bool, CommonError> {
		DocumentHandler::SaveDocumentLogic(&self.ApplicationHandle, self, uri).await
	}

	// Saves the document currently identified by `OriginalUri` to a new
	// location.
	async fn SaveDocumentAs(&self, original_uri:Url, new_target_uri:Option<Url>) -> Result<Option<Url>, CommonError> {
		// This is a complex operation that often involves User Interface interaction.
		// For now, we stub it. A full implementation would use the UiProvider to prompt
		// the user for a path.
		warn!("[DocumentProvider] SaveDocumentAs is not fully implemented.");
		Ok(None)
	}

	// Saves all currently dirty documents.
	async fn SaveAllDocuments(&self, include_untitled:bool) -> Result<Vec<bool>, CommonError> {
		// A full implementation would iterate `AppState.OpenDocuments` and save dirty
		// ones.
		warn!("[DocumentProvider] SaveAllDocuments is not fully implemented.");
		Ok(vec![])
	}

	// Applies a collection of content changes to the document at the given
	// URI.
	async fn ApplyDocumentChanges(
		&self,
		uri:Url,
		new_version_identifier:i64,
		changes_DTO_collection:Value,
	) -> Result<(), CommonError> {
		DocumentHandler::ApplyDocumentChangesLogic(
			&self.ApplicationHandle,
			uri,
			new_version_identifier,
			changes_DTO_collection,
		)
		.await
	}
}

impl Requires<Arc<dyn DocumentsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DocumentsProvider + Send + Sync> { Arc::new(self.clone()) }
}
