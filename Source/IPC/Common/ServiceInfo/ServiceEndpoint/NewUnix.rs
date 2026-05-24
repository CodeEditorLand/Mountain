//! `ServiceEndpoint::NewUnix`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(Path:impl Into<String>) -> Struct {
		Self {
			Protocol:"unix".to_string(),

			Address:String::new(),

			Port:0,

			Path:Some(Path.into()),
		}
	}
