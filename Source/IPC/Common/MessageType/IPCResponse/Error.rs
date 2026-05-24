//! `IPCResponse::Error`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(CorrelationId:impl Into<String>, Error:impl Into<String>) -> Struct {
		Self {
			CorrelationId:CorrelationId.into(),

			Data:serde_json::Value::Null,

			Success:false,

			Error:Some(Error.into()),

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}
