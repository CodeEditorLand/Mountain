//! `SecurityContext::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(user_id:String, roles:Vec<String>, permissions:Vec<String>, ip_address:String) -> Struct {
		Self { user_id, roles, permissions, ip_address, timestamp:std::time::SystemTime::now() }
	}
