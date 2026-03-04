//! # Hover Interface
//!
//! Defines types and traits for hover functionality.
//!
//! ## Responsibilities
//!
//! - Position and range types for document navigation
//! - Hover request/response DTOs
//! - Language-agnostic hover content definitions
//!
//! ## Architectural Role
//!
//! This module is part of the **Command layer**, providing type definitions
//! for the Hover language feature command.

use serde::{Deserialize, Serialize};

/// Position in a document (LSP-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
	/// Zero-based line number
	pub line:u32,
	/// Zero-based character offset within the line
	pub character:u32,
}

/// A range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
	/// Start of the range
	pub start:Position,
	/// End of the range
	pub end:Position,
}

/// Request payload for hover operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverRequest {
	/// The URI of the document to get hover for
	pub uri:String,
	/// The position in the document
	pub position:Position,
}

/// Represents the content of a hover result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum HoverContent {
	/// Plain text content
	PlainText(String),
	/// Markdown content
	Markdown(String),
	/// Structured content with language
	Markup {
		/// The content value
		value:String,
		/// The programming language (if applicable)
		language:Option<String>,
	},
}

/// Response from hover operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResponse {
	/// The hover contents
	pub contents:Vec<HoverContent>,
	/// Optional range this hover applies to
	#[serde(skip_serializing_if = "Option::is_none")]
	pub range:Option<Range>,
}

impl Default for HoverResponse {
	fn default() -> Self { Self { contents:Vec::new(), range:None } }
}

impl HoverResponse {
	/// Create a new hover response with content
	pub fn new(contents:Vec<HoverContent>) -> Self { Self { contents, range:None } }

	/// Create a hover response with content and range
	pub fn with_range(contents:Vec<HoverContent>, range:Range) -> Self { Self { contents, range:Some(range) } }
}
