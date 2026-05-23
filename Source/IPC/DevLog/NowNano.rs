
//! Wall-clock nanoseconds since UNIX epoch. Used as the start
//! tick for OTLP spans and per-IPC latency measurements.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn Fn() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64 }
