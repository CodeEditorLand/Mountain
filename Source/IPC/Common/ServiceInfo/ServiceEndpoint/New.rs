//! `ServiceEndpoint::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(Protocol:impl Into<String>, Address:impl Into<String>, Port:u16) -> Struct {
		Self { Protocol:Protocol.into(), Address:Address.into(), Port, Path:None }
	}
