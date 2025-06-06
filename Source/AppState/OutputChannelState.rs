
// Defines the data structure for representing the state of a single output
// channel.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

/// Represents the in-memory state of an output channel.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OutputChannelState {
	// The display name of the output channel.
	pub Name:String,
	// Optional language identifier for syntax highlighting of the channel's content.
	pub LanguageIdentifier:Option<String>,
	// The buffered content of the output channel.
	pub Buffer:String,
	// Whether the output channel is currently visible in the UI.
	pub Visible:bool,
}

impl OutputChannelState {
	/// Creates a new, empty `OutputChannelState`.
	pub fn New(Name:&str, LanguageIdentifier:Option<String>) -> Self {
		Self { Name:Name.to_string(), LanguageIdentifier, Buffer:String::new(), Visible:false }
	}
}
