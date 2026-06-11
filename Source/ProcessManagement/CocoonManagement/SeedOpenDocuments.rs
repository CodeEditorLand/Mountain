//! Seed Cocoon's `__textDocuments` with every file already open in the
//! workbench so extensions reading `workspace.textDocuments` synchronously
//! in `activate()` see the real editor state.

use std::sync::Arc;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(SideCarId:&str, Environment:&Arc<MountainEnvironment>) {
	// Seed Cocoon's `__textDocuments` with any files already open in the
	// workbench. Extensions that read `workspace.textDocuments` synchronously
	// in their `activate()` function (rust-analyzer, ESLint, TypeScript) must
	// see already-open editors rather than an empty array.
	let OpenDocs = Environment.ApplicationState.Feature.Documents.GetAll();

	if !OpenDocs.is_empty() {
		dev_log!(
			"exthost",
			"[CocoonManagement] Seeding {} open document(s) to Cocoon",
			OpenDocs.len()
		);

		for Doc in OpenDocs.values() {
			let Payload = serde_json::json!({
				"uri": Doc.URI.to_string(),
				"languageId": Doc.LanguageIdentifier,
				"version": Doc.Version,
				"lines": Doc.Lines,
			});

			let _ = crate::Vine::Client::SendNotification::Fn(
				SideCarId.to_string(),
				"$acceptModelAdded".to_string(),
				Payload,
			)
			.await;
		}
	}
}
