//! # OutputChannelStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single output
//! channel.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

/// Represents the complete state of a single output channel, including its
/// buffered content and visibility status.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OutputChannelStateDTO {
	pub Name:String,

	pub LanguageIdentifier:Option<String>,

	pub Buffer:String,

	pub IsVisible:bool,
}

impl OutputChannelStateDTO {
	/// Creates a new `OutputChannelStateDTO`.
	pub fn Create(Name:&str, LanguageIdentifier:Option<String>) -> Self {
		Self { Name:Name.to_string(), LanguageIdentifier, ..Default::default() }
	}
}
