//! # URLSerializer Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Serializes URL objects to JSON strings for data transfer and storage.
//!
//! ## ARCHITECTURAL ROLE
//! URLSerializer is part of the **Internal::Serialization** module,
//! providing URL serialization utilities.
//!
//! ## KEY COMPONENTS
//! - SerializeURL: Function to serialize URL to JSON value
//!
//! ## ERROR HANDLING
//! - Returns serde_json::Value for the URL string representation
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (debug).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient serialization
//! - Minimal overhead
//!
//! ## TODO
//! - [ ] Add URL validation before serialization
//! - [ ] Implement custom serialization formats
//! - [ ] Add performance metrics

use serde::Serializer;
use url::Url;

use crate::dev_log;

/// Serializes a URL to a JSON string value.
///
/// # Arguments
/// * `URLInstance` - The URL to serialize
/// * `SerializerInstance` - The serde serializer instance
///
/// # Returns
/// Result containing the serialized string or serialization error
///
/// # Behavior
/// - Converts URL to its string representation
/// - Uses the serializer to create a JSON string value
pub fn SerializeURL<S>(URLInstance:&Url, SerializerInstance:S) -> Result<S::Ok, S::Error>
where
	S: Serializer, {
	let url_string = URLInstance.as_str();

	dev_log!("ipc", "[URLSerializer] Serializing URL: {}", url_string);

	SerializerInstance.serialize_str(url_string)
}
