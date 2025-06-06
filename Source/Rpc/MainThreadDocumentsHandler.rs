// File: Rpc/MainThreadDocumentsHandler.rs
// Defines the RPC handler for document-related operations originating from the
// sidecar. This includes opening, creating, and saving documents.

use std::sync::Arc;

use Common::{DocumentsEffects, Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};
use url::Url;

use crate::{
	Handlers::{self, Documents::ParseUriFromComponentsParameter, ErrorUtils},
	Rpc::Args::Documents::{
		SaveAllArgument as SaveAllDocumentsArgument,
		TryOpenArgument as TryOpenDocumentArgument,
		TrySaveArgument as TrySaveDocumentArgument,
		TrySaveAsArgument as TrySaveDocumentAsArgument,
	},
	Runtime::AppRuntime,
}; // Assuming this function exists and is PascalCased

#[derive(Clone)]
pub struct MainThreadDocumentsHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadDocumentsHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Attempts to open an existing document or create a new one if content is
	/// provided.
	pub async fn TryOpenDocument(&self, Argument:TryOpenDocumentArgument) -> Result<Value, String> {
		let UriForLog = Argument
			.UriComponentsDto
			.get("external")
			.or_else(|| Argument.UriComponentsDto.get("path"));
		info!(
			"[Rpc DocumentsHandler] TryOpenDocument (DTO): URI(external/path)='{:?}', LangId='{:?}'",
			UriForLog, Argument.LanguageIdentifier
		);
		trace!(
			"[Rpc DocumentsHandler] TryOpenDocument Full URI DTO: {:?}",
			Argument.UriComponentsDto
		);

		let Effect =
			DocumentsEffects::OpenDocument(Argument.UriComponentsDto, Argument.LanguageIdentifier, Argument.Content);

		self.Runtime
			.Run(Effect)
			.await
			.map(|UrlResult| {
				json!({
					"$mid": 1,
					"scheme": UrlResult.scheme(),
					"path": UrlResult.path(),
					"external": UrlResult.to_string(),
					"fsPath": UrlResult.to_file_path().ok().as_ref().map_or_else(
						|| UrlResult.path().to_string(),
						|Path| Path.to_string_lossy().into_owned()
					)
				})
			})
			.map_err(|Error| {
				let OperationContext = format!("TryOpenDocument DTO for URI components: {:?}", UriForLog);
				ErrorUtils::MapCommonErrorToRpcString(Error, &OperationContext)
			})
	}

	/// Attempts to create a new untitled document.
	pub async fn TryCreateDocument(&self, Argument:TryOpenDocumentArgument) -> Result<Value, String> {
		// Note: TryOpenDocumentArgument is reused here as per the original `track.rs`
		// logic, where UriComponentsDto would be Value::Null for creation.
		info!(
			"[Rpc DocumentsHandler] TryCreateDocument (DTO): LangId='{:?}'",
			Argument.LanguageIdentifier
		);

		let Effect = DocumentsEffects::OpenDocument(
			Value::Null, // Indicates new document creation to the effect
			Argument.LanguageIdentifier,
			Argument.Content,
		);

		self.Runtime
			.Run(Effect)
			.await
			.map(|UrlResult| {
				json!({
					"$mid": 1,
					"scheme": UrlResult.scheme(),
					"path": UrlResult.path(),
					"external": UrlResult.to_string(),
					"fsPath": UrlResult.to_file_path().ok().as_ref().map_or_else(
						|| UrlResult.path().to_string(),
						|Path| Path.to_string_lossy().into_owned()
					)
				})
			})
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "TryCreateDocument DTO"))
	}

	/// Attempts to save an already open document.
	pub async fn TrySaveDocument(&self, Argument:TrySaveDocumentArgument) -> Result<Value, String> {
		let UriForLog = Argument
			.UriComponentsDto
			.get("external")
			.or_else(|| Argument.UriComponentsDto.get("path"));
		info!(
			"[Rpc DocumentsHandler] TrySaveDocument (DTO): URI(external/path)='{:?}'",
			UriForLog
		);
		trace!(
			"[Rpc DocumentsHandler] TrySaveDocument Full URI DTO: {:?}",
			Argument.UriComponentsDto
		);

		let UriToSave = ParseUriFromComponentsParameter(
			&Argument.UriComponentsDto,
			"TrySaveDocument RPC",
			"UriComponentsDto",
			None,
		)?;

		let Effect = DocumentsEffects::SaveDocument(UriToSave.clone());

		self.Runtime
			.Run(Effect)
			.await
			.map(|SuccessBool| json!(SuccessBool))
			.map_err(|Error| {
				let OperationContext = format!("TrySaveDocument DTO for {}", UriToSave);
				ErrorUtils::MapCommonErrorToRpcString(Error, &OperationContext)
			})
	}

	/// Attempts to save a document to a new location or with a new name.
	pub async fn TrySaveDocumentAs(&self, Argument:TrySaveDocumentAsArgument) -> Result<Value, String> {
		let OriginalUriForLog = Argument
			.OriginalUriComponentsDto
			.get("external")
			.or_else(|| Argument.OriginalUriComponentsDto.get("path"));
		let NewTargetUriForLog = Argument
			.NewTargetUriComponentsDto
			.as_ref()
			.and_then(|v| v.get("external").or_else(|| v.get("path")));
		info!(
			"[Rpc DocumentsHandler] TrySaveDocumentAs (DTO): OriginalURI='{:?}', NewTargetURI='{:?}'",
			OriginalUriForLog, NewTargetUriForLog
		);

		let OriginalUrl = ParseUriFromComponentsParameter(
			&Argument.OriginalUriComponentsDto,
			"TrySaveDocumentAs RPC (Original URI)",
			"OriginalUriComponentsDto",
			None,
		)?;

		let NewTargetUrlOption:Option<Url> = Argument
			.NewTargetUriComponentsDto
			.map(|TargetDto| {
				ParseUriFromComponentsParameter(
					&TargetDto,
					"TrySaveDocumentAs RPC (New Target URI)",
					"NewTargetUriComponentsDto",
					None,
				)
			})
			.transpose()?; // Propagate parsing error if any

		let Effect = DocumentsEffects::SaveDocumentAs(OriginalUrl.clone(), NewTargetUrlOption);

		self.Runtime
			.Run(Effect)
			.await
			.map(|NewUriOption| {
				NewUriOption.map_or(Value::Null, |NewUrl| {
					json!({
						"$mid": 1,
						"scheme": NewUrl.scheme(),
						"path": NewUrl.path(),
						"external": NewUrl.to_string(),
						"fsPath": NewUrl.to_file_path().ok().as_ref().map_or_else(
							|| NewUrl.path().to_string(),
							|Path| Path.to_string_lossy().into_owned()
						)
					})
				})
			})
			.map_err(|Error| {
				let OperationContext = format!("TrySaveDocumentAs DTO for {}", OriginalUrl);
				ErrorUtils::MapCommonErrorToRpcString(Error, &OperationContext)
			})
	}

	/// Saves all currently dirty documents.
	pub async fn SaveAll(&self, Argument:SaveAllDocumentsArgument) -> Result<Value, String> {
		info!(
			"[Rpc DocumentsHandler] SaveAll (DTO): IncludeUntitled={}",
			Argument.IncludeUntitled
		);
		let Effect = DocumentsEffects::SaveAllDocuments(Argument.IncludeUntitled);
		self.Runtime
			.Run(Effect)
			.await
			.map(|ResultsBoolVec| json!(ResultsBoolVec))
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "SaveAll DTO"))
	}
}
