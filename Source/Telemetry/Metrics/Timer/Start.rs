//! `Timer::Start`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::Telemetry::Metrics::GlobalRegistry;

pub fn Fn(Name:&str) -> Struct { Self { Name:Name.to_string(), Labels:HashMap::new(), Start:Instant::now() } }
