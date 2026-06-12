//! Error type returned by `FeatureFlagRegistry` operations.

#[derive(Debug, thiserror::Error)]
/// Enumeration for enum.
pub enum Enum {
	#[error("Feature flag not found: {0}")]
	NotFound(String),

	#[error("Feature flag already exists: {0}")]
	AlreadyExists(String),

	#[error("Feature flag error: {0}")]
	Other(String),
}
