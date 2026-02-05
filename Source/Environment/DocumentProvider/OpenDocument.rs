//! Document opening and content resolution logic.
//!
//! Handles opening documents from file:// URIs, custom scheme URIs (via sidecar
//! providers), and already-open documents.

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
	IPC::IPCProvider::IPCProvider,
};
use log::{error, info};
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use url::Url;

use crate::{
	ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO,
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Opens a document. If the URI scheme is not native (`file`), it attempts to
/// resolve the content from a registered sidecar provider
/// (`TextDocumentContentProvider`).
pub(super) async fn open_document(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	uri_components_dto: Value,
	language_identifier: Option<String>,
	content: Option<String>,
) -> Result<Url, CommonError> {
	let uri = Utility::GetURLFromURIComponentsDTO(&uri_components_dto)?;

	info!("[DocumentProvider] Opening document: {}", uri);

	// First, check if the document is already open.
	if let Some(existing_document) = environment
		.ApplicationState
		.OpenDocuments
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.get(uri.as_str())
	{
		info!("[DocumentProvider] Document {} is already open.", uri);

		match existing_document.ToDTO() {
			Ok(dto) => {
				if let Err(error) = environment.ApplicationHandle.emit("sky://documents/open", dto) {
					error!("[DocumentProvider] Failed to emit document open event: {}", error);
				}
			}
			Err(error) => {
				error!("[DocumentProvider] Failed to serialize existing document DTO: {}", error);
			}
		}

		return Ok(existing_document.URI.clone());
	}

	// Resolve the content based on the URI scheme.
	let file_content = if let Some(c) = content {
		c
	} else if uri.scheme() == "file" {
		let file_path = uri
			.to_file_path()
			.map_err(|_| CommonError::InvalidArgument {
				ArgumentName: "URI".into(),
				Reason: "Cannot convert non-file URI to path".into(),
			})?;

		let runtime = environment
			.ApplicationHandle
			.state::<Arc<ApplicationRunTime>>()
			.inner()
			.clone();

		let file_content_bytes = runtime.Run(ReadFile(file_path.clone())).await?;

		String::from_utf8(file_content_bytes)
			.map_err(|error| CommonError::FileSystemIO {
				Path: file_path,
				Description: error.to_string(),
			})?
	} else {
		// Custom scheme: attempt to resolve from a sidecar provider.
		info!(
			"[DocumentProvider] Non-native scheme '{}'. Attempting to resolve from sidecar.",
			uri.scheme()
		);

		let ipc_provider: Arc<dyn IPCProvider> = environment.Require();

		let rpc_result = ipc_provider
			.SendRequestToSideCar(
				// In a multi-host world, we'd look this up
				"cocoon-main".to_string(),
				"$provideTextDocumentContent".to_string(),
				json!([uri_components_dto]),
				10000,
			)
			.await?;

		rpc_result
			.as_str()
			.map(String::from)
			.ok_or_else(|| CommonError::IPCError {
				Description: format!(
					"Failed to get valid string content for custom URI scheme '{}'",
					uri.scheme()
				),
			})?
	};

	// The rest of the flow is the same for all schemes.
	let new_document = DocumentStateDTO::Create(uri.clone(), language_identifier, file_content)?;

	let dto_for_notification = new_document.ToDTO()?;

	environment
		.ApplicationState
		.OpenDocuments
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.insert(uri.to_string(), new_document);

	if let Err(error) = environment
		.ApplicationHandle
		.emit("sky://documents/open", dto_for_notification.clone())
	{
		error!("[DocumentProvider] Failed to emit document open event: {}", error);
	}

	crate::Environment::DocumentProvider::Notifications::notify_model_added(
		environment,
		&dto_for_notification,
	)
	.await;

	Ok(uri)
}
