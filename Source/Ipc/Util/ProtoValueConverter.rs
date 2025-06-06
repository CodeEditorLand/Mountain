// File: Ipc/Util/ProtoValueConverter.rs
// Defines utility functions for converting between `serde_json::Value`
// and the `JsonValueWrapper` used in Prost-generated gRPC messages.

#![allow(non_snake_case, non_camel_case_types)]

use log::error;
use serde_json::Value as JsonValue;

use crate::Ipc::Generated::VineGrpcPb::JsonValueWrapper;

/// Converts a `serde_json::Value` into an `Option<JsonValueWrapper>`.
/// `Value::Null` is converted to `Some(JsonValueWrapper { value: Value::Null
/// })` to distinguish it from a completely absent value (`None`).
pub fn JsValueToProtoValue(JsValue:JsonValue) -> Result<Option<JsonValueWrapper>, String> {
	if JsValue.is_null() {
		// Explicitly wrap null, as `None` might imply an omitted field.
		return Ok(Some(JsonValueWrapper { value:JsonValue::Null }));
	}
	// All other JSON value types can be wrapped directly.
	Ok(Some(JsonValueWrapper { value:JsValue }))
}

/// Converts an `Option<JsonValueWrapper>` back into a `serde_json::Value`.
/// If the option is `None`, it returns `Value::Null`.
pub fn ProtoValueToJsValue(ProtoValueOption:Option<JsonValueWrapper>) -> Result<JsonValue, String> {
	match ProtoValueOption {
		Some(Wrapper) => Ok(Wrapper.value),
		None => Ok(JsonValue::Null),
	}
}
