pub mod Start;
pub mod WithLabel;
pub mod StopAndRecord;

use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::Telemetry::Metrics::GlobalRegistry;

pub struct Struct {
	Name:String,

	Labels:HashMap<String, String>,

	Start:Instant,
}
