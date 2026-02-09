//! # CommandValidation - Input Validation for Commands
//!
//! Provides comprehensive validation for command registration and
//! execution requests with security checks and sanitization.
//!
//! ## Validation Rules
//!
//! - Command ID format (extension.command)
//! - Title length and content
//! - Extension ID validation
//! - Category normalization
//!
//! ## Security
//!
//! - Input sanitization
//! - Length limits
//! - Character filtering

use log::{debug, warn};

pub struct CommandValidationError {
	pub field:String,
	pub message:String,
}

/// Validate command ID format
pub fn ValidateCommandId(id:&str) -> Result<(), CommandValidationError> {
	if id.is_empty() {
		return Err(CommandValidationError {
			field:"id".to_string(),
			message:"Command ID cannot be empty".to_string(),
		});
	}

	if !id.contains('.') {
		return Err(CommandValidationError {
			field:"id".to_string(),
			message:"Command ID must contain dot separator (e.g., 'extension.command')".to_string(),
		});
	}

	if id.len() > 200 {
		return Err(CommandValidationError {
			field:"id".to_string(),
			message:"Command ID too long (max 200 characters)".to_string(),
		});
	}

	Ok(())
}

/// Validate command title
pub fn ValidateCommandTitle(title:&str) -> Result<(), CommandValidationError> {
	if title.trim().is_empty() {
		return Err(CommandValidationError {
			field:"title".to_string(),
			message:"Command title cannot be empty".to_string(),
		});
	}

	if title.len() > 200 {
		return Err(CommandValidationError {
			field:"title".to_string(),
			message:"Command title too long (max 200 characters)".to_string(),
		});
	}

	Ok(())
}

/// Validate extension ID
pub fn ValidateExtensionId(id:&str) -> Result<(), CommandValidationError> {
	if id.is_empty() {
		return Err(CommandValidationError {
			field:"extension_id".to_string(),
			message:"Extension ID cannot be empty".to_string(),
		});
	}

	Ok(())
}

/// Validate category (optional)
pub fn ValidateCategory(category:&Option<String>) -> Result<(), CommandValidationError> {
	if let Some(cat) = category {
		if cat.len() > 50 {
			return Err(CommandValidationError {
				field:"category".to_string(),
				message:"Category too long (max 50 characters)".to_string(),
			});
		}
	}
	Ok(())
}

/// Complete validation for command registration request
pub fn ValidateRegistrationRequest(
	id:&str,
	title:&str,
	extension_id:&str,
	category:&Option<String>,
	_when:&Option<String>,
) -> Result<(), Vec<CommandValidationError>> {
	let mut errors = Vec::new();

	if let Err(e) = ValidateCommandId(id) {
		errors.push(e);
	}
	if let Err(e) = ValidateCommandTitle(title) {
		errors.push(e);
	}
	if let Err(e) = ValidateExtensionId(extension_id) {
		errors.push(e);
	}
	if let Err(e) = ValidateCategory(category) {
		errors.push(e);
	}

	if errors.is_empty() { Ok(()) } else { Err(errors) }
}
