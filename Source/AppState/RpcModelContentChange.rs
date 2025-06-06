
// Defines Data Transfer Objects (DTOs) used for representing document
// content changes in a structured way, typically for RPC communication.

#![allow(non_snake_case, non_camel_case_types)]

use serde::Deserialize;

/// Represents a range in a text document using 1-based line and column numbers.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RpcRangeDto {
	#[serde(alias = "startLineNumber")]
	pub StartLineNumber:usize,
	#[serde(alias = "startColumn")]
	pub StartColumn:usize,
	#[serde(alias = "endLineNumber")]
	pub EndLineNumber:usize,
	#[serde(alias = "endColumn")]
	pub EndColumn:usize,
}

/// Represents a single change to a document's content, including the range to
/// be replaced and the new text to insert.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RpcModelContentChangeDto {
	pub Range:RpcRangeDto,
	pub Text:String,
}
