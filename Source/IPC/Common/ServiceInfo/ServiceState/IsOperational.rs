//! `ServiceState::IsOperational`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> bool { matches!(self, Enum::Running | Enum::Degraded | Enum::Starting) }
