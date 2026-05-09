#![allow(non_snake_case)]

//! IPC response: correlation ID, payload, success flag, optional error
//! string, and timestamp. Built through `Success` / `Error` constructors
//! that stamp the timestamp from the chrono UTC clock.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub CorrelationId:String,

	pub Data:serde_json::Value,

	pub Success:bool,

	pub Error:Option<String>,

	pub Timestamp:u64,
}

impl Struct {

	pub fn Success(CorrelationId:impl Into<String>, Data:serde_json::Value) -> Self {

		Self {

			CorrelationId:CorrelationId.into(),

			Data,

			Success:true,

			Error:None,

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}

	pub fn Error(CorrelationId:impl Into<String>, Error:impl Into<String>) -> Self {

		Self {

			CorrelationId:CorrelationId.into(),

			Data:serde_json::Value::Null,

			Success:false,

			Error:Some(Error.into()),

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}
}
