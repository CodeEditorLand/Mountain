#![allow(non_snake_case)]

//! Number of currently-active broadcast subscribers. Diagnostic; useful
//! for validating that subscribers haven't leaked.

use crate::Vine::Client::Shared;

pub fn Fn() -> usize { Shared::NOTIFICATION_BROADCAST.receiver_count() }
