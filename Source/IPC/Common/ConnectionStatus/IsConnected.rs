//! `ConnectionStatus::IsConnected`

use super::Struct;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> bool { This.state == ConnectionState::Connected }
