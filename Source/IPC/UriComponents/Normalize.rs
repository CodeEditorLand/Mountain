#![allow(non_snake_case)]

//! Normalise an `extensionLocation` (or any similar) field that arrives
//! as either a URL string, a pre-built `UriComponents` object (possibly
//! already tagged), or is missing / null. The output is always a
//! tagged object with the five URI fields.

use serde_json::Value;

use crate::IPC::UriComponents::{FromFilePath, FromUrl, StampMidUri};

pub fn Fn(Raw:Option<&Value>) -> Value {
	match Raw {
		Some(Value::Object(Map)) if Map.contains_key("scheme") => StampMidUri::Fn(Value::Object(Map.clone())),
		Some(Value::String(Url)) => FromUrl::Fn(Url),
		_ => FromFilePath::Fn("/extensions/unknown"),
	}
}
