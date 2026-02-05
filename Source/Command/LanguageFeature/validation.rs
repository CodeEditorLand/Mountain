//! # LanguageFeature - Validation
//!
//! Validation helper for language feature requests

use serde_json;

/// Validates language feature request parameters.
pub(super) fn validate_language_feature_request(request_type: &str, uri: &str, position: &serde_json::Value) -> Result<(), String> {
	if uri.is_empty() {
		return Err(format!("Empty URI for {} request", request_type));
	}

	// Validate position format
	if let Some(line) = position.get("line") {
		if !line.is_u64() {
			return Err(format!("Invalid line position for {} request", request_type));
		}
	} else {
		return Err(format!("Missing line position for {} request", request_type));
	}

	if let Some(character) = position.get("character") {
		if !character.is_u64() {
			return Err(format!("Invalid character position for {} request", request_type));
		}
	} else {
		return Err(format!("Missing character position for {} request", request_type));
	}

	Ok(())
}
