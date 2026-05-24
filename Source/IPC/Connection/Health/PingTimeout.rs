//! `Health::PingTimeout`

use super::Struct;
use super::Types::ConnectionHandle;
use crate::dev_log;

pub fn Fn(This:&Struct) -> std::time::Duration { This.PingTimeout }
