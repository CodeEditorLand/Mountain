// File: AppState/UrlSerdeHelper.rs
// Defines a helper module for serializing and deserializing `url::Url`
// instances with Serde, allowing them to be easily stored in structs.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{self, Deserialize, Deserializer, Serializer};
use url::Url;

/// Serializes a `&url::Url` to its string representation.
pub fn serialize<S>(UrlInstance:&Url, SerializerInstance:S) -> Result<S::Ok, S::Error>
where
	S: Serializer, {
	SerializerInstance.serialize_str(UrlInstance.as_str())
}

/// Deserializes a string into a `url::Url`.
pub fn deserialize<'de, D>(DeserializerInstance:D) -> Result<Url, D::Error>
where
	D: Deserializer<'de>, {
	let StringValue = String::deserialize(DeserializerInstance)?;
	Url::parse(&StringValue).map_err(serde::de::Error::custom)
}
