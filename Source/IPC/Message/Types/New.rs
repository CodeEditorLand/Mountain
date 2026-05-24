//! `Types::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(connected:bool) -> Struct { Self { connected } }
