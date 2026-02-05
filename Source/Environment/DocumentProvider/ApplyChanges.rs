//! Document change application logic.
//!
//! Handles applying incremental text edits to documents via LSP-style
//! DidChangeTextDocument notifications.

use CommonLibrary::Error::CommonError::CommonError;
use log::{trace, warn};
use serde_json::Value;
use url::Url;

use crate::Environment::Utility;

/// Applies a collection of content changes to a document.
pub(super) async fn apply_document_changes(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	uri: Url,
	new_version_identifier: i64,
	changes_dto_collection: Value,
	_is_dirty_after_change: bool,
	_is_undoing: bool,
	_is_redoing: bool,
) -> Result<(), CommonError> {
	trace!("[DocumentProvider] Applying changes to document: {}", uri);

	{
		let mut open_documents_guard = environment
			.ApplicationState
			.OpenDocuments
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(document) = open_documents_guard.get_mut(uri.as_str()) {
			document.ApplyChanges(new_version_identifier, &changes_dto_collection)?;
		} else {
			warn!("[DocumentProvider] Received changes for unknown document: {}", uri);

			return Ok(());
		}
	}

	super::Notifications::notify_model_changed(
		environment,
		&uri,
		new_version_identifier,
		changes_dto_collection,
	)
	.await;

	Ok(())
}
