//! `IPCResponse::Success`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(CorrelationId:impl Into<String>, Data:serde_json::Value) -> Struct {
		Self {
			CorrelationId:CorrelationId.into(),

			Data,

			Success:true,

			Error:None,

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}
