//! # RPCModelContentChangeDTO
//!
//! Defines the Data Transfer Objects for representing text changes in a
//! document, compatible with VS Code's RPC protocol for text synchronization.

#![allow(non_snake_case, non_camel_case_types)]

use serde::Deserialize;

/// Represents a line and column-based range in a text document.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RPCRangeDTO {
	pub StartLineNumber:usize,
	pub StartColumn:usize,
	pub EndLineNumber:usize,
	pub EndColumn:usize,
}

/// Represents a single text change operation, including the range to be
/// replaced and the new text to insert. This is part of a collection sent when
/// a document is edited.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RPCModelContentChangeDTO {
	pub Range:RPCRangeDTO,
	pub Text:String,
}
