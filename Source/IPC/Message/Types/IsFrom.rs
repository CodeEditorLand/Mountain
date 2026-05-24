//! `Types::IsFrom`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct, sender:&str) -> bool { This.sender.as_deref() == Some(sender) }
