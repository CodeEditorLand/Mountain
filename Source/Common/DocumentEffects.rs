// File: Common/DocumentEffect.rs
// Defines the DocumentProvider trait and associated effects for document
// management. This provides a standardized way to open, save, and modify
// documents within the application's environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use url::Url;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can provide document management capabilities.
#[async_trait]
pub trait DocumentProvider: Environment {
	/// Opens a document specified by a URI components DTO. Can also create a
	/// new untitled document.
	async fn OpenDocument(
		&self,
		UriComponentsDto:Value,
		LanguageIdentifier:Option<String>,
		Content:Option<String>,
	) -> Result<Url, CommonError>;

	/// Saves an existing document.
	async fn SaveDocument(&self, Uri:Url) -> Result<bool, CommonError>;

	/// Saves a document to a new location.
	async fn SaveDocumentAs(&self, OriginalUri:Url, NewTargetUri:Option<Url>) -> Result<Option<Url>, CommonError>;

	/// Saves all currently open and dirty documents.
	async fn SaveAllDocuments(&self, IncludeUntitled:bool) -> Result<Vec<bool>, CommonError>;

	/// Applies a set of content changes to a document.
	async fn ApplyDocumentChanges(
		&self,
		Uri:Url,
		NewVersionIdentifier:i64,
		ChangesDtoCollection:Value,
		IsDirtyAfterChange:bool,
		IsUndoing:bool,
		IsRedoing:bool,
	) -> Result<(), CommonError>;
}

/// Creates an effect to open a document.
pub fn OpenDocument<RuntimeAccessType>(
	UriComponentsDto:Value,
	LanguageIdentifier:Option<String>,
	Content:Option<String>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Url>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DocumentProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let UriDtoClone = UriComponentsDto.clone();
		let LanguageIdentifierClone = LanguageIdentifier.clone();
		let ContentClone = Content.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn DocumentProvider> = Environment.require();
			Provider.OpenDocument(UriDtoClone, LanguageIdentifierClone, ContentClone).await
		})
	}))
}

/// Creates an effect to save a document.
pub fn SaveDocument<RuntimeAccessType>(Uri:Url) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, bool>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DocumentProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let UriClone = Uri.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn DocumentProvider> = Environment.require();
			Provider.SaveDocument(UriClone).await
		})
	}))
}

/// Creates an effect to save a document to a new location.
pub fn SaveDocumentAs<RuntimeAccessType>(
	OriginalUri:Url,
	NewTargetUri:Option<Url>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<Url>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DocumentProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OriginalUriClone = OriginalUri.clone();
		let NewTargetUriClone = NewTargetUri.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn DocumentProvider> = Environment.require();
			Provider.SaveDocumentAs(OriginalUriClone, NewTargetUriClone).await
		})
	}))
}

/// Creates an effect to save all dirty documents.
pub fn SaveAllDocuments<RuntimeAccessType>(
	IncludeUntitled:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<bool>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DocumentProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn DocumentProvider> = Environment.require();
			Provider.SaveAllDocuments(IncludeUntitled).await
		})
	}))
}

/// Creates an effect to apply changes to a document.
pub fn ApplyDocumentChanges<RuntimeAccessType>(
	Uri:Url,
	NewVersionIdentifier:i64,
	ChangesDtoCollection:Value,
	IsDirtyAfterChange:bool,
	IsUndoing:bool,
	IsRedoing:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn DocumentProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let UriClone = Uri.clone();
		let ChangesClone = ChangesDtoCollection.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn DocumentProvider> = Environment.require();
			Provider
				.ApplyDocumentChanges(
					UriClone,
					NewVersionIdentifier,
					ChangesClone,
					IsDirtyAfterChange,
					IsUndoing,
					IsRedoing,
				)
				.await
		})
	}))
}
