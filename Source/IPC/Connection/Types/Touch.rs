//! `Types::Touch`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct) { This.last_used = std::time::SystemTime::now(); }
