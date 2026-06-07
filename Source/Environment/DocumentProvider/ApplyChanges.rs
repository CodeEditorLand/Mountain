//! Document change application logic.
//!
//! Implements `ApplyDocumentChanges` for `MountainEnvironment`. Receives an
//! LSP-style `DidChangeTextDocument` payload (a JSON array of range-based
//! text edits), applies it to the in-memory `DocumentStateDTO` stored in
//! `ApplicationState::Feature::Documents::OpenDocuments`, then forwards the
//! same payload to Cocoon via `notify_model_changed` so the extension host's
//! model mirrors the workbench state.
//!
//! Unknown URIs (document not in `OpenDocuments`) are silently ignored with
//! a `warn:` log - this can legitimately happen during the brief window
//! between a document being closed on the Sky side and the last in-flight
//! change notification arriving from Cocoon.

use CommonLibrary::Error::CommonError::CommonError;

use serde_json::Value;

use url::Url;

use crate::{Environment::Utility, dev_log};

/// Applies a collection of content changes to a document.
pub(super) async fn apply_document_changes(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	uri:Url,

	new_version_identifier:i64,

	changes_dto_collection:Value,

	_is_dirty_after_change:bool,

	_is_undoing:bool,

	_is_redoing:bool,
) -> Result<(), CommonError> {

	dev_log!("model", "[DocumentProvider] Applying changes to document: {}", uri);

	{
		let mut open_documents_guard = environment
			.ApplicationState
			.Feature
			.Documents
			.OpenDocuments
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		if let Some(document) = open_documents_guard.get_mut(uri.as_str()) {
			document.ApplyChanges(new_version_identifier, &changes_dto_collection)?;
		} else {
			dev_log!(
				"model",

				"warn: [DocumentProvider] Received changes for unknown document: {}",

				uri
			);

			return Ok(());
		}
	}

	super::Notifications::notify_model_changed(environment, &uri, new_version_identifier, changes_dto_collection).await;

	Ok(())
}
