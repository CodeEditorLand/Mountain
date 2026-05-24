//! `ConnectionHandle::Age`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn(This:&Struct) -> Duration { This.created_at.elapsed() }
