
// Implements the `DocumentProvider` trait for the `MountainEnvironment`.
// This file connects abstract document effects to the concrete logic
// in the application's document handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{DocumentEffect::DocumentProvider, Environment::Requires, Errors::CommonError};
use async_trait::async_trait;
use log::info;
use serde_json::Value;
use url::Url;

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	/// Opens a document.
	async fn OpenDocument(
		&self,
		UriComponentsDto:Value,
		LanguageIdentifierOverrideOption:Option<String>,
		InitialContentOption:Option<String>,
	) -> Result<Url, CommonError> {
		info!(
			"[Environment DocumentsProvider] OpenDocument: Uri='{:?}', LangOverride='{:?}', HasContent={}",
			UriComponentsDto.get("external").or_else(|| UriComponentsDto.get("path")),
			LanguageIdentifierOverrideOption,
			InitialContentOption.is_some()
		);

		Handlers::Documents::HandleOpenDocumentEffectLogic(
			self.AppHandle.clone(),
			self.clone(),
			UriComponentsDto,
			LanguageIdentifierOverrideOption,
			InitialContentOption,
		)
		.await
	}

	/// Saves a document.
	async fn SaveDocument(&self, UriToSave:Url) -> Result<bool, CommonError> {
		info!("[Environment DocumentsProvider] SaveDocument: Uri='{}'", UriToSave);
		Handlers::Documents::HandleSaveDocumentEffectLogic(self.AppHandle.clone(), self.clone(), UriToSave).await
	}

	/// Saves a document to a new location.
	async fn SaveDocumentAs(
		&self,
		OriginalUri:Url,
		NewUriTargetOption:Option<Url>,
	) -> Result<Option<Url>, CommonError> {
		info!(
			"[Environment DocumentsProvider] SaveDocumentAs: Original='{}', Target='{:?}'",
			OriginalUri, NewUriTargetOption
		);
		Handlers::Documents::HandleSaveDocumentAsEffectLogic(
			self.AppHandle.clone(),
			self.clone(),
			OriginalUri,
			NewUriTargetOption,
		)
		.await
	}

	/// Saves all currently dirty documents.
	async fn SaveAllDocuments(&self, IncludeUntitled:bool) -> Result<Vec<bool>, CommonError> {
		info!(
			"[Environment DocumentsProvider] SaveAllDocuments: IncludeUntitled={}",
			IncludeUntitled
		);
		Handlers::Documents::HandleSaveAllDocumentsEffectLogic(self.AppHandle.clone(), self.clone(), IncludeUntitled)
			.await
	}

	/// Applies content changes to a document.
	async fn ApplyDocumentChanges(
		&self,
		UriToChange:Url,
		NewVersionIdentifier:i64,
		ChangesDtoCollectionValue:Value,
		IsDirtyAfterChange:bool,
		IsUndoingOperation:bool,
		IsRedoingOperation:bool,
	) -> Result<(), CommonError> {
		info!(
			"[Environment DocumentsProvider] ApplyDocumentChanges: Uri='{}', NewVersion={}",
			UriToChange, NewVersionIdentifier,
		);
		Handlers::Documents::HandleApplyDocumentChangesEffectLogic(
			self.AppHandle.clone(),
			self.clone(),
			UriToChange,
			NewVersionIdentifier,
			ChangesDtoCollectionValue,
			IsDirtyAfterChange,
			IsUndoingOperation,
			IsRedoingOperation,
		)
		.await
	}
}

impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}
