//! # Hover Function
//!
//! Implements hover functionality for language features.
//!
//! ## Responsibilities
//!
//! - Handle hover requests from the frontend
//! - Delegate to language feature providers
//! - Transform provider responses to standard format
//!
//! ## Architectural Role
//!
//! This module is part of the **Command layer**, providing the actual
//! hover functionality that bridges the frontend to the language service.
//!
//! ## Design
//!
//! - Single async function as the entry point
//! - Validates input before processing
//! - Delegates to providers for actual implementation
//! - Returns standardized response

use serde_json::Value;
use tauri::{AppHandle, Wry};
use url::Url;

use crate::{
	Command::Hover::Interface::{
		HoverRequest::Struct as HoverRequest,
		HoverResponse::Struct as HoverResponse,
		Position::Struct as Position,
	},
	dev_log,
};

/// Validates a hover request
fn ValidateRequest(uri:&str, position:&Value) -> Result<HoverRequest, String> {
	// Parse URI
	let document_uri = Url::parse(uri).map_err(|e| format!("Invalid URI: {}", e))?;

	// Parse position from JSON value
	let position_dto:Position =
		serde_json::from_value(position.clone()).map_err(|e| format!("Invalid position: {}", e))?;

	Ok(HoverRequest { uri:document_uri.to_string(), position:position_dto })
}

/// Provides hover information at the given document position.
///
/// This function is the main entry point for the hover command,
/// called by the Tauri command dispatcher.
///
/// # Arguments
///
/// * `application_handle` - The Tauri application handle
/// * `uri` - The URI of the document
/// * `position` - The position in the document to get hover for
///
/// # Returns
///
/// Returns a `HoverResponse` containing the hover contents, or an error string.
pub async fn Hover(application_handle:AppHandle<Wry>, uri:String, position:Value) -> Result<HoverResponse, String> {
	dev_log!("commands", "[Hover] Providing hover for: {} at {:?}", uri, position);

	// Validate request
	let request = ValidateRequest(&uri, &position)?;

	// Get the document URI
	let document_uri = Url::parse(&request.uri).map_err(|e| format!("Failed to parse URI: {}", e))?;

	// Delegate to the provider implementation
	// Note: This is a stub - actual implementation would call the provider
	let _result = ProvideHover(document_uri, request.position).await?;

	// For now, return an empty response
	// DEPENDENCY: Full hover implementation requires provider registry in
	// ApplicationState and provider invocation via RPC to language server
	Ok(HoverResponse::default())
}

/// Internal implementation to get hover from a provider.
///
/// This would typically invoke the language feature provider registry
/// to find an appropriate provider for the document.
async fn ProvideHover(_uri:Url, _position:Position) -> Result<HoverResponse, String> {
	// DEPENDENCY: Provider invocation requires:
	// 1. Provider registry lookup in ApplicationState by document URI
	// 2. RPC call to language server via CocoonService
	// 3. Result transformation to HoverResponse

	dev_log!("commands", "[Hover] Calling provider for hover information");

	Ok(HoverResponse::default())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_request_valid() {
		let uri = "file:///test.rs";
		let position = serde_json::json!({
			"line": 10,
			"character": 5
		});

		let result = ValidateRequest(uri, &position);
		assert!(result.is_ok());

		let request = result.unwrap();
		assert_eq!(request.uri, uri);
		assert_eq!(request.position.line, 10);
		assert_eq!(request.position.character, 5);
	}

	#[test]
	fn test_validate_request_invalid_uri() {
		let uri = "not-a-valid-uri";
		let position = serde_json::json!({
			"line": 10,
			"character": 5
		});

		let result = ValidateRequest(uri, &position);
		assert!(result.is_err());
	}

	#[test]
	fn test_validate_request_invalid_position() {
		let uri = "file:///test.rs";
		let position = serde_json::json!({
			"not_a_position": true
		});

		let result = ValidateRequest(uri, &position);
		assert!(result.is_err());
	}

	#[test]
	fn test_hover_response_default() {
		let response = HoverResponse::default();
		assert!(response.contents.is_empty());
		assert!(response.range.is_none());
	}

	#[test]
	fn test_hover_response_with_contents() {
		use crate::Command::Hover::Interface::HoverContent::Enum as HoverContent;

		let contents = vec![HoverContent::PlainText("Test hover".to_string())];
		let response = HoverResponse::new(contents);

		assert_eq!(response.contents.len(), 1);
		assert!(response.range.is_none());
	}
}
