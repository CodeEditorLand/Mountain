// @module OutputChannelStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single output channel.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

// Represents the complete state of a single output channel, including its
// buffered content.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OutputChannelStateDTO {
	pub Name:String,
	pub LanguageIdentifier:Option<String>,
	pub Buffer:String,
	pub Visible:bool,
}

impl OutputChannelStateDTO {
	// Creates a new `OutputChannelStateDTO`.
	pub fn New(name:&str, language_identifier:Option<String>) -> Self {
		Self {
			Name:name.to_string(),
			LanguageIdentifier:language_identifier,
			..Default::default()
		}
	}
}
