//! `Types::Description`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> &'static str {
		if This.connected {
			"Connected to Mountain"
		} else {
			"Disconnected from Mountain"
		}
	}
