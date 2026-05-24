//! `ConnectionStatus::IsHealthy`

use super::Struct;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> bool { matches!(This.state, ConnectionState::Connected | ConnectionState::Connecting) }
