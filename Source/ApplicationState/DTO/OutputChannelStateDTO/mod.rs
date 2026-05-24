pub mod Create;
pub mod Append;
pub mod Clear;
pub mod GetBufferSize;
pub mod GetFormattedBufferSize;
pub mod SetVisibility;

use serde::{Deserialize, Serialize};

/// Represents the complete state of a single output channel, including its
/// buffered content and visibility status.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// Channel display name
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Name:String,

	/// Language identifier for syntax highlighting
	#[serde(skip_serializing_if = "Option::is_none")]
	pub LanguageIdentifier:Option<String>,

	/// Buffered output content
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Buffer:String,

	/// Whether the channel is currently visible in UI
	pub IsVisible:bool,
}
