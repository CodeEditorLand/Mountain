//! `Types::IsHealthy`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> bool { This.health_score > 50.0 && This.error_count < 5 }
