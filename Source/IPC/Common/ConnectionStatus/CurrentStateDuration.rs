//! `ConnectionStatus::CurrentStateDuration`

use super::Struct;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> Duration { This.state_since.elapsed() }
