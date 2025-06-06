// File: Rpc/MainThreadWindowHandler.rs
// Defines the RPC handler for window-related actions initiated by the sidecar,
// such as focusing the main window or handling URI opening requests.

use std::sync::Arc;

use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, State, Window, Wry}; // Added Window
use url::Url;

use crate::{
	Handlers::ErrorUtils,
	Rpc::Args::Window::{AsExternalUriArgument, OpenUriArgument},
	Runtime::AppRuntime,
};

#[derive(Clone)]
pub struct MainThreadWindowHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>, // Kept for consistency, though not used in focusWindow
}

impl MainThreadWindowHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Sets focus to the main application window.
	/// `_ArgumentsValue` is currently unused but kept for signature consistency
	/// if other window actions are added.
	pub async fn FocusWindow(&self, _ArgumentsValue:Value) -> Result<Value, String> {
		info!("[Rpc MainThreadWindowHandler] FocusWindow (DTO flow)");

		if let Some(MainWindow) = self.ApplicationHandle.get_webview_window("main") {
			MainWindow.set_focus().map_err(|TauriError| {
				ErrorUtils::RpcErrorString(
					format!("Failed to focus main window: {}", TauriError),
					Some("EWINDOW_FOCUS_FAIL"),
				)
			})?;
			Ok(Value::Null)
		} else {
			Err(ErrorUtils::RpcErrorString(
				"Main window not found for FocusWindow operation".to_string(),
				Some("ENOWINDOW_FOCUS_OP"),
			))
		}
	}

	/// Opens a URI, potentially externally.
	pub async fn OpenUri(&self, Argument:OpenUriArgument) -> Result<Value, String> {
		let UriToOpenString = Argument
			.UriComponentsDto
			.get("external")
			.and_then(Value::as_str)
			.or_else(|| Argument.UriComponentsDto.get("path").and_then(Value::as_str))
			.unwrap_or("MISSING_URI_IN_OPENURI_DTO");

		info!(
			"[Rpc MainThreadWindowHandler] OpenUri (DTO): URI='{}', Options='{:?}'",
			UriToOpenString, Argument.Options
		);

		// Validate and parse the URI
		let TargetUrl = Url::parse(UriToOpenString).map_err(|ParseError| {
			ErrorUtils::RpcErrorString(
				format!("Invalid URI in OpenUri DTO: {}. URI: '{}'", ParseError, UriToOpenString),
				Some("EBADURI_OPENURI"),
			)
		})?;

		// Delegate to Tauri's shell API to open the URL.
		// The options from DTO (like allowExternalSchemes) are more for VS Code's
		// internal opener routing, which is simplified here to a direct shell open.
		match self.ApplicationHandle.shell().open(TargetUrl.as_str(), None) {
			Ok(_) => {
				debug!(
					"[Rpc MainThreadWindowHandler] OpenUri: Successfully launched external URI '{}'",
					TargetUrl.as_str()
				);
				Ok(Value::Bool(true)) // VS Code's openExternal returns a boolean
			},
			Err(TauriError) => {
				error!(
					"[Rpc MainThreadWindowHandler] OpenUri: Failed to open external URI '{}': {}",
					TargetUrl.as_str(),
					TauriError
				);
				Ok(Value::Bool(false)) // Indicate failure
			},
		}
	}

	/// Converts a URI to its external form, if applicable.
	pub async fn AsExternalUri(&self, Argument:AsExternalUriArgument) -> Result<Value, String> {
		let UriToConvertString = Argument
			.UriComponentsDto
			.get("external")
			.and_then(Value::as_str)
			.or_else(|| Argument.UriComponentsDto.get("path").and_then(Value::as_str))
			.unwrap_or("MISSING_URI_IN_ASEXTERNALURI_DTO");

		info!(
			"[Rpc MainThreadWindowHandler] AsExternalUri (DTO): URI='{}', Options='{:?}'",
			UriToConvertString, Argument.Options
		);

		// For this shim, if it's a common web scheme, we assume it's already external.
		// For file schemes, we also return it as-is, as "external" form for local files
		// is usually the file URI itself. More complex transformations (e.g. for
		// remote) are not handled here.
		let TargetUrl = Url::parse(UriToConvertString).map_err(|ParseError| {
			ErrorUtils::RpcErrorString(
				format!(
					"Invalid URI in AsExternalUri DTO: {}. URI: '{}'",
					ParseError, UriToConvertString
				),
				Some("EBADURI_ASEXTERNALURI"),
			)
		})?;

		let Scheme = TargetUrl.scheme().to_lowercase();
		if Scheme == "http" || Scheme == "https" || Scheme == "mailto" || Scheme == "file" {
			debug!(
				"[Rpc MainThreadWindowHandler] AsExternalUri: URI '{}' is suitable as external.",
				TargetUrl.as_str()
			);
			return Ok(Argument.UriComponentsDto); // Return the original DTO as it's considered external
		}

		warn!(
			"[Rpc MainThreadWindowHandler] AsExternalUri: Cannot convert URI scheme '{}' to an external form. \
			 Returning original.",
			Scheme
		);
		Ok(Argument.UriComponentsDto)
	}
}
