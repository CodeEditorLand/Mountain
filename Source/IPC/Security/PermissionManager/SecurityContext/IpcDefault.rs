//! `SecurityContext::IpcDefault`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn() -> Struct {
		Self {
			user_id:"ipc-connection".to_string(),

			roles:vec!["user".to_string()],

			permissions:vec![],

			ip_address:"127.0.0.1".to_string(),

			timestamp:std::time::SystemTime::now(),
		}
	}
