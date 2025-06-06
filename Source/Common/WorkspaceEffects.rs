// File: Common/WorkspaceEffect.rs
// Defines traits and effects for interacting with the workspace.
// This includes managing folders, trust, files, and applying batch edits.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use serde_json::Value;
use url::Url;

use crate::LanguageFeatureEffect::WorkspaceEditDto; // Assuming this is the correct path for the DTO
use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that provide workspace-level information and
/// actions.
#[async_trait]
pub trait WorkspaceProvider: Environment {
	/// Retrieves information about all folders in the current workspace.
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError>;
	/// Retrieves information about the specific workspace folder that contains
	/// the given URI.
	async fn GetWorkspaceFolderInfo(&self, UriToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError>;
	/// Gets the name of the current workspace.
	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError>;
	/// Gets the path to the workspace configuration file (e.g.,
	/// `.code-workspace`).
	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError>;
	/// Checks if the current workspace is trusted.
	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError>;
	/// Prompts the user to grant or deny trust to the current workspace.
	async fn RequestWorkspaceTrust(&self, Options:Option<Value>) -> Result<bool, CommonError>;
	/// Finds files within the workspace matching given glob patterns.
	async fn FindFilesInWorkspace(
		&self,
		IncludePatternDto:Value,
		ExcludePatternDto:Option<Value>,
		MaxResultCount:Option<usize>,
		UseIgnoreFiles:bool,
		FollowSymbolicLinks:bool,
	) -> Result<Vec<Url>, CommonError>;
	/// Opens a file in the editor.
	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError>;
}

/// A trait for environments that can apply a `WorkspaceEdit`.
#[async_trait]
pub trait WorkspaceEditApplier: Environment {
	/// Applies a `WorkspaceEdit`, which can contain a mix of text edits and
	/// file operations.
	async fn ApplyWorkspaceEdit(&self, EditDto:WorkspaceEditDto) -> Result<bool, CommonError>;
}

/// Creates an effect to get information about all workspace folders.
pub fn GetWorkspaceFolders<RuntimeAccessType>()
-> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<(Url, String, usize)>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.GetWorkspaceFoldersInfo().await
		})
	}))
}

/// Creates an effect to get information about a specific workspace folder.
pub fn GetWorkspaceFolderInfo<RuntimeAccessType>(
	UriToMatch:Url,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<(Url, String, usize)>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let UriClone = UriToMatch.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.GetWorkspaceFolderInfo(UriClone).await
		})
	}))
}

/// Creates an effect to get the workspace name.
pub fn GetWorkspaceName<RuntimeAccessType>() -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<String>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.GetWorkspaceName().await
		})
	}))
}

/// Creates an effect to get the workspace configuration file path.
pub fn GetWorkspaceConfigurationPath<RuntimeAccessType>()
-> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<PathBuf>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.GetWorkspaceConfigurationPath().await
		})
	}))
}

/// Creates an effect to check if the workspace is trusted.
pub fn IsWorkspaceTrusted<RuntimeAccessType>() -> ActionEffect<Arc<RuntimeAccessType>, CommonError, bool>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.IsWorkspaceTrusted().await
		})
	}))
}

/// Creates an effect to request workspace trust.
pub fn RequestWorkspaceTrust<RuntimeAccessType>(
	OptionsDto:Option<Value>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, bool>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OptionsClone = OptionsDto.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.RequestWorkspaceTrust(OptionsClone).await
		})
	}))
}

/// Creates an effect to find files in the workspace.
pub fn FindFilesInWorkspace<RuntimeAccessType>(
	IncludePatternDto:Value,
	ExcludePatternDto:Option<Value>,
	MaxResultCount:Option<usize>,
	UseIgnoreFiles:bool,
	FollowSymbolicLinks:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<Url>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let IncludeClone = IncludePatternDto.clone();
		let ExcludeClone = ExcludePatternDto.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider
				.FindFilesInWorkspace(IncludeClone, ExcludeClone, MaxResultCount, UseIgnoreFiles, FollowSymbolicLinks)
				.await
		})
	}))
}

/// Creates an effect to open a file.
pub fn OpenFile<RuntimeAccessType>(Path:PathBuf) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn WorkspaceProvider> = Environment.require();
			Provider.OpenFile(PathClone).await
		})
	}))
}

/// Creates an effect to apply a workspace edit.
pub fn ApplyWorkspaceEdit<RuntimeAccessType>(
	EditDto:WorkspaceEditDto,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, bool>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn WorkspaceEditApplier>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let EditDtoClone = EditDto.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Applier:Arc<dyn WorkspaceEditApplier> = Environment.require();
			Applier.ApplyWorkspaceEdit(EditDtoClone).await
		})
	}))
}
