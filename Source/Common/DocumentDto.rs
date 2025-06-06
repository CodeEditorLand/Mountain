// File: Common/DocumentDto.rs
// Defines a general-purpose Data Transfer Object (DTO) for document-related
// operations. This struct aggregates various optional fields to accommodate
// different actions like opening, saving, and applying changes, reducing the
// need for multiple, highly-specific DTOs.

#![allow(non_snake_case, non_camel_case_types)]

use serde_json::Value;
use url::Url;

// This DTO seems to be for internal use or a very generic container.
// Individual effects (in DocumentEffects.rs) use more specific parameters.
// This is kept for structural completeness based on the provided file list.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DocumentDto {
	pub Uri:Url,
	pub VersionIdentifier:Option<i64>,
	pub LanguageIdentifier:Option<String>,
	pub Content:Option<String>,
	// Represents a collection of text changes, typically an array of change DTOs.
	pub Changes:Option<Value>,
	pub Eol:Option<String>,
	pub IsDirty:Option<bool>,
	pub IsUndoing:Option<bool>,
	pub IsRedoing:Option<bool>,
	// Used specifically for "save as" operations.
	pub NewTargetUri:Option<Url>,
	// Used specifically for "save all" operations.
	pub IncludeUntitled:Option<bool>,
}
