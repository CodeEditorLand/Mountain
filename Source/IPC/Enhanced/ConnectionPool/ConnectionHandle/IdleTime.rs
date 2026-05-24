//! `ConnectionHandle::IdleTime`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn(This:&Struct) -> Duration { This.last_used.elapsed() }
