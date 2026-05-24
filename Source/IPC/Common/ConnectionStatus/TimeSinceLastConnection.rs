//! `ConnectionStatus::TimeSinceLastConnection`

use super::Struct;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> Option<Duration> { This.last_connected.map(|t| t.elapsed()) }
