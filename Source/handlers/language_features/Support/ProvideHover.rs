use Common::{error::CommonError, language_feature::dto::*};
use log::debug;
use serde_json::json;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

/// @module ProvideHover (LanguageFeatures/Support)
/// @description Logic for invoking the hover provider.
use crate::{AppState::AppState::AppState, vine};

pub async fn ProvideHoverLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	DocumentUri:Url,
	Position:PositionDto,
) -> Result<Option<HoverResultDto>, CommonError> {
	debug!("[ProvideHoverLogic] Requesting hover for {}", DocumentUri);

	// 1. Find the best provider for this document from AppState.
	// This is complex logic involving matching the document selector.
	// For now, we'll assume we found one.
	let ProviderHandle = 1; // Placeholder
	let SidecarId = "cocoon-main".to_string(); // Placeholder

	// 2. Make the RPC call to Cocoon.
	let Response = vine::client::SendRequest(
		SidecarId,
		"$provideHover".to_string(),
		json!([ProviderHandle, DocumentUri.to_string(), Position]),
		5000, // 5-second timeout
	)
	.await?;

	// 3. Deserialize and return the result.
	serde_json::from_value(Response).map_err(|e| CommonError::SerdeError { Description:e.to_string() })
}
