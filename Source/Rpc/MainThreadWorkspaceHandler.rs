// File: Rpc/MainThreadWorkspaceHandler.rs
// Defines the RPC handler for workspace-related operations requested by the
// sidecar. This includes resolving workspace folders and finding files within
// the workspace.

use std::path::PathBuf; // Added for file_path_to_uri_components_dto
use std::sync::Arc;

use Common::WorkspaceEffects; // Assuming this path
use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};
use url::Url; // Added for Url type

use crate::{
	Handlers::{self, ErrorUtils},
	Rpc::{
		Args::Workspace::{FindFilesArgument, ResolveFolderArgument as ResolveWorkspaceFolderArgument},
		file_path_to_uri_components_dto,
	},
	Runtime::AppRuntime,
}; // Assuming this utility function

#[derive(Clone)]
pub struct MainThreadWorkspaceHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadWorkspaceHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Resolves information about a workspace folder given a URI that is part
	/// of it.
	pub async fn ResolveWorkspaceFolder(&self, Argument:ResolveWorkspaceFolderArgument) -> Result<Value, String> {
		let UriToResolveString = Argument
			.UriComponentsDto
			.get("external")
			.and_then(Value::as_str)
			.or_else(|| Argument.UriComponentsDto.get("path").and_then(Value::as_str))
			.unwrap_or("MISSING_URI_IN_RESOLVEWSFOLDER_DTO");

		info!(
			"[Rpc WorkspaceHandler] ResolveWorkspaceFolder (DTO): URI='{}'",
			UriToResolveString
		);

		let UriToResolve = Url::parse(UriToResolveString).map_err(|ParseError| {
			ErrorUtils::RpcErrorString(
				format!(
					"Invalid URI in ResolveWorkspaceFolder DTO: {}. URI: '{}'",
					ParseError, UriToResolveString
				),
				Some("EBADURI_RESOLVEWSFOLDER"),
			)
		})?;

		let Effect = WorkspaceEffects::GetWorkspaceFolderInfo(UriToResolve);
		self.Runtime
			.Run(Effect)
			.await
			.map(|OptionalFolderInfo| {
				json!(OptionalFolderInfo.map(|(FolderUrl, Name, Index)| {
					json!({
						"uri": file_path_to_uri_components_dto(&PathBuf::from(FolderUrl.path())), // Convert Url to PathBuf then to DTO
						"name": Name,
						"index": Index
					})
				}))
			})
			.map_err(|CommonErrorValue| {
				ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "ResolveWorkspaceFolder DTO")
			})
	}

	/// Finds files within the workspace based on include/exclude patterns and
	/// options.
	pub async fn FindFiles(&self, Argument:FindFilesArgument) -> Result<Value, String> {
		debug!(
			"[Rpc WorkspaceHandler] FindFiles (DTO): Include='{:?}', Exclude='{:?}', Options='{:?}'",
			Argument.Include, Argument.Exclude, Argument.Options
		);

		// The WorkspaceEffects::FindFiles effect expects direct Value parameters, not
		// the DTO struct. We need to serialize the DTO parts back into the expected
		// Value structure.
		let IncludeValue = serde_json::to_value(&Argument.Include).map_err(|SerializationError| {
			ErrorUtils::RpcInternalErrorString(format!(
				"Failed to serialize Include DTO for FindFiles: {}",
				SerializationError
			))
		})?;

		let ExcludeValueOption = Argument
			.Exclude
			.map(|ExcludePattern| serde_json::to_value(&ExcludePattern))
			.transpose()
			.map_err(|SerializationError| {
				ErrorUtils::RpcInternalErrorString(format!(
					"Failed to serialize Exclude DTO for FindFiles: {}",
					SerializationError
				))
			})?;

		let OptionsValueOption = Argument
			.Options
			.map(|OptionsDto| serde_json::to_value(&OptionsDto))
			.transpose()
			.map_err(|SerializationError| {
				ErrorUtils::RpcInternalErrorString(format!(
					"Failed to serialize Options DTO for FindFiles: {}",
					SerializationError
				))
			})?;

		let Effect = WorkspaceEffects::FindFilesInWorkspace(
			IncludeValue,
			ExcludeValueOption,
			OptionsValueOption
				.as_ref()
				.and_then(|OptsVal| OptsVal.get("maxResults").and_then(Value::as_u64).map(|Max| Max as usize)),
			OptionsValueOption
				.as_ref()
				.and_then(|OptsVal| OptsVal.get("useIgnoreFiles").and_then(Value::as_bool))
				.unwrap_or(true), // Default from track.rs
			OptionsValueOption
				.as_ref()
				.and_then(|OptsVal| OptsVal.get("followSymlinks").and_then(Value::as_bool))
				.unwrap_or(false), // Default from track.rs
		);

		self.Runtime
			.Run(Effect)
			.await
			.map(|UrlVec| {
				json!(
					UrlVec
						.into_iter()
						.map(|UrlItem| file_path_to_uri_components_dto(&PathBuf::from(UrlItem.path())))
						.collect::<Vec<_>>()
				)
			})
			.map_err(|CommonErrorValue| ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "FindFiles DTO"))
	}
}
