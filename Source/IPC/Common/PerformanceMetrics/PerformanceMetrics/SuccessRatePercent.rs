//! `PerformanceMetrics::SuccessRatePercent`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn(This:&Struct) -> f64 { This.SuccessRate() * 100.0 }
