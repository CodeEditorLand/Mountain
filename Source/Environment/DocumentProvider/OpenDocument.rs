//! Document opening and content resolution logic.
//!
//! Implements `OpenDocument` for `MountainEnvironment`. Resolution order:
//!
//! 1. **Already open** - if the URI is already in `OpenDocuments`, re-emits
//!    `sky://documents/open` so the Sky workbench focuses the existing tab
//!    and returns immediately without a disk read.
//! 2. **Caller-supplied content** - if the `content` argument is `Some`,
//!    it is used verbatim (used by untitled / virtual documents).
//! 3. **`file://` URI** - content is read from disk via `ApplicationRunTime`.
//! 4. **Custom scheme** - content is fetched from the Cocoon sidecar via
//!    `$provideTextDocumentContent` RPC (10 s timeout). This covers schemes
//!    like `git:`, `output:`, `vscode-notebook-cell:`, etc.
//!
//! On success, a new `DocumentStateDTO` is inserted into `OpenDocuments`,
//! `sky://documents/open` is emitted to the Sky workbench, and
//! `$acceptModelAdded` is sent to Cocoon.

use std::sync::Arc;

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
	IPC::{IPCProvider::IPCProvider, SkyEvent::SkyEvent},
};
use serde_json::{Value, json};
// `Emitter` was previously imported here for the now-replaced
// direct `.emit(...)` calls; emit is now done via `LogSkyEmit`
// which carries the trait import internally. `Manager` remains
// because `.state::<…>()` below depends on it.
use tauri::Manager;
use url::Url;

use crate::{
	ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO,
	Environment::Utility,
	IPC::SkyEmit::LogSkyEmit,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Opens a document. If the URI scheme is not native (`file`), it attempts to
/// resolve the content from a registered sidecar provider
/// (`TextDocumentContentProvider`).
pub(super) async fn open_document(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	uri_components_dto:Value,

	language_identifier:Option<String>,

	content:Option<String>,
) -> Result<Url, CommonError> {
	let uri = Utility::UriParsing::GetURLFromURIComponentsDTO(&uri_components_dto)?;

	dev_log!("model", "[DocumentProvider] Opening document: {}", uri);

	// First, check if the document is already open.
	if let Some(existing_document) = environment
		.ApplicationState
		.Feature
		.Documents
		.OpenDocuments
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.get(uri.as_str())
	{
		dev_log!("model", "[DocumentProvider] Document {} is already open.", uri);

		match existing_document.ToDTO() {
			Ok(dto) => {
				if let Err(error) = LogSkyEmit(&environment.ApplicationHandle, SkyEvent::DocumentsOpen.AsStr(), dto) {
					dev_log!(
						"model",
						"error: [DocumentProvider] Failed to emit document open event: {}",
						error
					);
				}
			},

			Err(error) => {
				dev_log!(
					"model",
					"error: [DocumentProvider] Failed to serialize existing document DTO: {}",
					error
				);
			},
		}

		return Ok(existing_document.URI.clone());
	}

	// Resolve the content based on the URI scheme.
	let file_content = if let Some(c) = content {
		c
	} else if uri.scheme() == "file" {
		let file_path = uri.to_file_path().map_err(|_| {
			CommonError::InvalidArgument {
				ArgumentName:"URI".into(),
				Reason:"Cannot convert non-file URI to path".into(),
			}
		})?;

		let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		let file_content_bytes = runtime.Run(ReadFile(file_path.clone())).await?;

		String::from_utf8(file_content_bytes)
			.map_err(|error| CommonError::FileSystemIO { Path:file_path, Description:error.to_string() })?
	} else {
		// Custom scheme: attempt to resolve from a sidecar provider.
		dev_log!(
			"model",
			"[DocumentProvider] Non-native scheme '{}'. Attempting to resolve from sidecar.",
			uri.scheme()
		);

		let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

		let rpc_result = ipc_provider
			.SendRequestToSideCar(
				// In a multi-host world, we'd look this up
				"cocoon-main".to_string(),
				"$provideTextDocumentContent".to_string(),
				json!([uri_components_dto]),
				10000,
			)
			.await?;

		rpc_result.as_str().map(String::from).ok_or_else(|| {
			CommonError::IPCError {
				Description:format!("Failed to get valid string content for custom URI scheme '{}'", uri.scheme()),
			}
		})?
	};

	// The rest of the flow is the same for all schemes.
	let new_document = DocumentStateDTO::Create(uri.clone(), language_identifier, file_content)?;

	let dto_for_notification = new_document.ToDTO()?;

	environment
		.ApplicationState
		.Feature
		.Documents
		.OpenDocuments
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.insert(uri.to_string(), new_document);

	if let Err(error) = LogSkyEmit(
		&environment.ApplicationHandle,
		SkyEvent::DocumentsOpen.AsStr(),
		dto_for_notification.clone(),
	) {
		dev_log!(
			"model",
			"error: [DocumentProvider] Failed to emit document open event: {}",
			error
		);
	}

	crate::Environment::DocumentProvider::Notifications::notify_model_added(environment, &dto_for_notification).await;

	Ok(uri)
}
