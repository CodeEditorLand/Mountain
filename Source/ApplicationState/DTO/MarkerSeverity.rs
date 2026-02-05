//! # MarkerSeverity
//!
//! # RESPONSIBILITY
//! - Defines severity levels for diagnostic markers
//! - Compatible with VS Code's marker severity values
//!
//! # FIELDS
//! - Error: 8
//! - Warning: 4
//! - Information: 2
//! - Hint: 1

/// Marker severity levels (aligned with VS Code)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MarkerSeverity {
	/// Something not allowed by the rules of a language
	Error = 8,
	/// Something suspicious but allowed
	Warning = 4,
	/// Something to inform about but not a problem
	Information = 2,
	/// Something to help improvement
	Hint = 1,
}
