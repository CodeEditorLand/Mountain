//! # URLDeserializer Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Deserializes JSON strings to URL objects for data transfer and storage.
//!
//! ## ARCHITECTURAL ROLE
//! URLDeserializer is part of the **Internal::Serialization** module,
//! providing URL deserialization utilities.
//!
//! ## KEY COMPONENTS
//! - DeserializeURL: Function to deserialize JSON to URL
//!
//! ## ERROR HANDLING
//! - Returns Result with URL or String error on parse failure
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (debug).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient deserialization
//! - Proper error handling
//!
//! ## TODO
//! - [ ] Add URL validation after deserialization
//! - [ ] Implement custom error recovery
//! - [ ] Add performance metrics

use serde::{Deserializer, de::Deserialize};
use url::Url;

use crate::dev_log;

/// Deserializes a JSON string value to a URL.
/// # Arguments
/// * `DeserializerInstance` - The serde deserializer instance
/// # Returns
/// Result containing the parsed URL or deserialization error
/// # Behavior
/// - Deserializes a string value
/// - Parses the string as a URL
/// - Returns parse error as custom deserialization error
pub fn Fn<'de, D>(DeserializerInstance:D) -> Result<Url, D::Error>
where
	D: Deserializer<'de>, {
	let string_value = String::deserialize(DeserializerInstance)?;

	dev_log!("ipc", "[URLDeserializer] Deserializing URL: {}", string_value);

	Url::parse(&string_value).map_err(serde::de::Error::custom)
}
