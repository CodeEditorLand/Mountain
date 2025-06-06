// File: Environment/WorkspaceProvider.rs
// Implements the `WorkspaceProvider` and `WorkspaceEditApplier` traits for the
// `MountainEnvironment`. This file connects abstract workspace effects to the
// concrete logic in the application's handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	DocumentEffect::DocumentProvider,
	Environment::Requires,
	Errors::CommonError,
	FsEffect::FsWriter,
	LanguageFeatureEffect::{
		FileEditTypeDto,
		WorkspaceCellEditDto,
		WorkspaceEditDto,
		WorkspaceFileEditDto,
		WorkspaceTextEditDto,
	},
	WorkspaceEffect::{WorkspaceEditApplier, WorkspaceProvider},
};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use url::Url;

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	/// Retrieves information about all folders in the current workspace.
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		trace!("[Environment WorkspaceProvider] GetWorkspaceFoldersInfo");
		Handlers::Workspace::HandleGetWorkspaceFoldersInfoEffectLogic(self.AppHandle.clone()).await
	}

	/// Retrieves information for the folder containing the given URI.
	async fn GetWorkspaceFolderInfo(&self, UriToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		debug!("[Environment WorkspaceProvider] GetWorkspaceFolderInfo for: {}", UriToMatch);
		Handlers::Workspace::HandleGetWorkspaceFolderInfoEffectLogic(self.AppHandle.clone(), UriToMatch).await
	}

	/// Gets the name of the current workspace.
	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError> {
		debug!("[Environment WorkspaceProvider] GetWorkspaceName");
		Handlers::Workspace::HandleGetWorkspaceNameEffectLogic(self.AppHandle.clone()).await
	}

	/// Gets the path to the workspace's configuration file.
	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		debug!("[Environment WorkspaceProvider] GetWorkspaceConfigurationPath");
		Handlers::Workspace::HandleGetWorkspaceConfigurationPathEffectLogic(self.AppHandle.clone()).await
	}

	/// Checks if the current workspace is trusted by the user.
	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError> {
		debug!("[Environment WorkspaceProvider] IsWorkspaceTrusted");
		Handlers::Workspace::HandleIsWorkspaceTrustedEffectLogic(self.AppHandle.clone()).await
	}

	/// Prompts the user to grant or deny trust to the workspace.
	async fn RequestWorkspaceTrust(&self, Options:Option<Value>) -> Result<bool, CommonError> {
		info!(
			"[Environment WorkspaceProvider] RequestWorkspaceTrust with options: {:?}",
			Options
		);
		// This handler needs the environment itself to potentially show a UI dialog.
		Handlers::Workspace::HandleRequestWorkspaceTrustEffectLogic(self.AppHandle.clone(), self.clone(), Options).await
	}

	/// Finds files in the workspace based on include/exclude patterns.
	async fn FindFilesInWorkspace(
		&self,
		IncludePatternDto:Value,
		ExcludePatternDto:Option<Value>,
		MaxResults:Option<usize>,
		UseIgnoreFiles:bool,
		FollowSymlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!(
			"[Environment WorkspaceProvider] FindFilesInWorkspace: Include='{:?}', Exclude='{:?}'",
			IncludePatternDto, ExcludePatternDto
		);
		Handlers::Workspace::HandleFindFilesInWorkspaceEffectLogic(
			self.AppHandle.clone(),
			IncludePatternDto,
			ExcludePatternDto,
			MaxResults,
			UseIgnoreFiles,
			FollowSymlinks,
		)
		.await
	}

	/// Opens a file in the editor from a workspace context.
	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError> {
		info!("[Environment WorkspaceProvider] OpenFile: {}", Path.display());
		// This handler needs the environment to perform the document opening operation.
		Handlers::Workspace::HandleOpenFileEffectLogic(self.AppHandle.clone(), self.clone(), Path).await
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	/// Applies a `WorkspaceEdit`, which is a collection of file and text edits.
	async fn ApplyWorkspaceEdit(&self, EditDto:WorkspaceEditDto) -> Result<bool, CommonError> {
		info!(
			"[Environment WorkspaceEditApplier] Applying WorkspaceEdit with {} edits.",
			EditDto.EditList.len()
		);

		let DocumentProviderInstance:Arc<dyn DocumentProvider> = self.require();
		let FsWriterInstance:Arc<dyn FsWriter> = self.require();

		for (Index, EditEntryValue) in EditDto.EditList.iter().enumerate() {
			trace!(
				"[Environment WorkspaceEditApplier] Processing edit entry #{}: Type {:?}",
				Index,
				EditEntryValue.get("_type")
			);

			let EditTypeNumber = EditEntryValue.get("_type").and_then(Value::as_u64);
			let EditTypeDtoOption:Option<FileEditTypeDto> =
				EditTypeNumber.and_then(|ValueU64| serde_json::from_value(Value::from(ValueU64)).ok());

			match EditTypeDtoOption {
				Some(FileEditTypeDto::Text) | Some(FileEditTypeDto::Snippet) => {
					let TextOperation = serde_json::from_value::<WorkspaceTextEditDto>(EditEntryValue.clone())
						.map_err(|e| {
							CommonError::InvalidArg {
								ArgumentName:"text_edit_entry".to_string(),
								Reason:format!("Deserialize WorkspaceTextEditDto failed: {}", e),
							}
						})?;
					let TargetUri = Handlers::Documents::ParseUriFromComponentsParameter(
						&TextOperation.Resource,
						"ApplyWorkspaceEdit (Text)",
						"resource",
						None,
					)
					.map_err(|ErrorString| {
						CommonError::InvalidArg { ArgumentName:"resource_uri".to_string(), Reason:ErrorString }
					})?;
					let SingleEditOperationValue = TextOperation.Edit;
					let RpcModelContentChange = json!({
						"range": SingleEditOperationValue.get("range").cloned().unwrap_or(Value::Null),
						"text": SingleEditOperationValue.get("text").cloned().unwrap_or(Value::Null),
						"eol": SingleEditOperationValue.get("eol").cloned(),
					});
					let ChangesArrayValue = Value::Array(vec![RpcModelContentChange]);
					let VersionIdentifierForApply = TextOperation.VersionIdentifier.map(|v| v as i64).unwrap_or(-1);
					info!("[Environment WorkspaceEditApplier] Applying TextEdit to: {}", TargetUri);
					DocumentProviderInstance
						.ApplyDocumentChanges(
							TargetUri.clone(),
							VersionIdentifierForApply,
							ChangesArrayValue,
							true,
							false,
							false,
						)
						.await?;
				},
				Some(FileEditTypeDto::File) => {
					let FileOperation = serde_json::from_value::<WorkspaceFileEditDto>(EditEntryValue.clone())
						.map_err(|e| {
							CommonError::InvalidArg {
								ArgumentName:"file_edit_entry".to_string(),
								Reason:format!("Deserialize WorkspaceFileEditDto failed: {}", e),
							}
						})?;
					let OldUrlOption = FileOperation.OldUri.as_ref().and_then(|v| {
						Handlers::Documents::ParseUriFromComponentsParameter(
							v,
							"ApplyWorkspaceEdit (File Old)",
							"old_uri",
							None,
						)
						.ok()
					});
					let NewUrlOption = FileOperation.NewUri.as_ref().and_then(|v| {
						Handlers::Documents::ParseUriFromComponentsParameter(
							v,
							"ApplyWorkspaceEdit (File New)",
							"new_uri",
							None,
						)
						.ok()
					});

					let Overwrite = FileOperation
						.Options
						.as_ref()
						.and_then(|o| o.get("overwrite").and_then(Value::as_bool))
						.unwrap_or(false);
					let Recursive = FileOperation
						.Options
						.as_ref()
						.and_then(|o| o.get("recursive").and_then(Value::as_bool))
						.unwrap_or(false);
					let IgnoreIfNotExists = FileOperation
						.Options
						.as_ref()
						.and_then(|o| o.get("ignoreIfNotExists").and_then(Value::as_bool))
						.unwrap_or(false);

					if let (Some(OldUri), Some(NewUri)) = (OldUrlOption.as_ref(), NewUrlOption.as_ref()) {
						info!(
							"[Environment WorkspaceEditApplier] Applying File Rename: {} -> {}",
							OldUri, NewUri
						);
						if OldUri.scheme() != "file" || NewUri.scheme() != "file" {
							return Err(CommonError::NotImplemented { FeatureName:"Rename for non-file URIs".into() });
						}
						FsWriterInstance
							.Rename(&PathBuf::from(OldUri.path()), &PathBuf::from(NewUri.path()), Overwrite)
							.await?;
					} else if let Some(NewUri) = NewUrlOption.as_ref() {
						info!("[Environment WorkspaceEditApplier] Applying File Create: {}", NewUri);
						if NewUri.scheme() != "file" {
							return Err(CommonError::NotImplemented { FeatureName:"Create for non-file URIs".into() });
						}
						FsWriterInstance
							.WriteFile(&PathBuf::from(NewUri.path()), Vec::new(), true, Overwrite)
							.await?;
					} else if let Some(OldUri) = OldUrlOption.as_ref() {
						info!("[Environment WorkspaceEditApplier] Applying File Delete: {}", OldUri);
						if OldUri.scheme() != "file" {
							return Err(CommonError::NotImplemented { FeatureName:"Delete for non-file URIs".into() });
						}
						let PathToDelete = PathBuf::from(OldUri.path());
						if IgnoreIfNotExists && !tokio::fs::try_exists(&PathToDelete).await.unwrap_or(false) {
							debug!(
								"[Environment WorkspaceEditApplier] Delete skipped for non-existent {} \
								 (ignoreIfNotExists=true)",
								OldUri
							);
						} else {
							FsWriterInstance.Delete(&PathToDelete, Recursive, false).await?;
						}
					} else {
						return Err(CommonError::InvalidArg {
							ArgumentName:"file_edit_entry".to_string(),
							Reason:"File operation DTO missing OldUri and NewUri.".to_string(),
						});
					}
				},
				Some(FileEditTypeDto::Cell)
				| Some(FileEditTypeDto::CellReplace)
				| Some(FileEditTypeDto::CellMetadata)
				| Some(FileEditTypeDto::DocumentMetadata) => {
					let CellOperationDetail = serde_json::from_value::<WorkspaceCellEditDto>(EditEntryValue.clone());
					warn!(
						"[Environment WorkspaceEditApplier] Notebook cell edit application is STUBBED for type: {:?}. \
						 Edit detail (if parsed): {:?}",
						EditTypeDtoOption,
						CellOperationDetail.ok()
					);
				},
				None => {
					warn!(
						"[Environment WorkspaceEditApplier] Unknown or missing _type in WorkspaceEditDto entry: {:?}. \
						 Edit: {:?}",
						EditTypeNumber, EditEntryValue
					);
					return Err(CommonError::InvalidArg {
						ArgumentName:"edit_entry._type".to_string(),
						Reason:"Unknown or missing edit _type".to_string(),
					});
				},
			}
		}
		Ok(true)
	}
}

impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}
