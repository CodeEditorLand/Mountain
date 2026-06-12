//! # OutputChannelStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for output channel state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track output channel lifecycle and content
//!
//! # FIELDS
//! - Name: Channel display name
//! - LanguageIdentifier: Language for syntax highlighting
//! - Buffer: Buffered output content
//! - IsVisible: Channel visibility status

use serde::{Deserialize, Serialize};

/// Maximum channel name length
const MAX_CHANNEL_NAME_LENGTH:usize = 128;

/// Maximum language identifier length
const MAX_LANGUAGE_ID_LENGTH:usize = 128;

/// Maximum buffer size per channel (prevents memory exhaustion)
/// Set to 10MB to prevent unbounded memory growth from excessive output
/// accumulation.
const MAX_BUFFER_SIZE:usize = 10_000_000;

/// the complete state of a single output channel, including its
/// buffered content and visibility status.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputChannelStateDTO {
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

impl OutputChannelStateDTO {
	/// Creates a new `OutputChannelStateDTO` with validation.
	/// # Arguments
	/// * `Name` - Channel name
	/// * `LanguageIdentifier` - Optional language identifier
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn Create(Name:&str, LanguageIdentifier:Option<String>) -> Result<Self, String> {
		// Validate name length
		if Name.len() > MAX_CHANNEL_NAME_LENGTH {
			return Err(format!(
				"Channel name exceeds maximum length of {} bytes",
				MAX_CHANNEL_NAME_LENGTH
			));
		}

		// Validate language identifier length
		if let Some(LangID) = &LanguageIdentifier {
			if LangID.len() > MAX_LANGUAGE_ID_LENGTH {
				return Err(format!(
					"Language identifier exceeds maximum length of {} bytes",
					MAX_LANGUAGE_ID_LENGTH
				));
			}
		}

		Ok(Self { Name:Name.to_string(), LanguageIdentifier, Buffer:String::new(), IsVisible:false })
	}

	/// Appends content to the buffer with size validation.
	/// # Arguments
	/// * `Content` - Content to append
	/// # Returns
	/// Result indicating success or error if buffer would exceed limit
	pub fn Append(&mut self, Content:&str) -> Result<(), String> {
		let NewSize = self.Buffer.len() + Content.len();

		if NewSize > MAX_BUFFER_SIZE {
			return Err(format!("Buffer would exceed maximum size of {} bytes", MAX_BUFFER_SIZE));
		}

		self.Buffer.push_str(Content);

		Ok(())
	}

	/// Clears the buffer content.
	pub fn Clear(&mut self) { self.Buffer.clear(); }

	/// Returns the current buffer size in bytes.
	pub fn GetBufferSize(&self) -> usize { self.Buffer.len() }

	/// Returns the current buffer size as a human-readable string.
	pub fn GetFormattedBufferSize(&self) -> String { FormatBytes(self.Buffer.len()) }

	/// Sets the visibility status.
	/// # Arguments
	/// * `IsVisible` - New visibility status
	pub fn SetVisibility(&mut self, IsVisible:bool) { self.IsVisible = IsVisible; }
}

/// Formats a byte count into a human-readable string.
fn FormatBytes(Bytes:usize) -> String {
	const UNITS:&[&str] = &["B", "KB", "MB", "GB"];

	if Bytes == 0 {
		return "0 B".to_string();
	}

	let mut Size = Bytes as f64;

	let mut MutIndex = 0usize;

	while Size >= 1024.0 && MutIndex < UNITS.len() - 1 {
		Size /= 1024.0;

		MutIndex += 1;
	}

	format!("{:.2} {}", Size, UNITS[MutIndex])
}
