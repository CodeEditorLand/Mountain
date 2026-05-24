//! `OutputChannelStateDTO::Append`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, Content:&str) -> Result<(), String> {
		let NewSize = This.Buffer.len() + Content.len();

		if NewSize > MAX_BUFFER_SIZE {
			return Err(format!("Buffer would exceed maximum size of {} bytes", MAX_BUFFER_SIZE));
		}

		This.Buffer.push_str(Content);

		Ok(())
	}
