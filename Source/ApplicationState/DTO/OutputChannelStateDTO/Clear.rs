//! `OutputChannelStateDTO::Clear`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct) { This.Buffer.clear(); }
