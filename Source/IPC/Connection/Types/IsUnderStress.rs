//! `Types::IsUnderStress`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> bool { This.Utilization() > 80.0 || This.HealthPercentage() < 70.0 }
