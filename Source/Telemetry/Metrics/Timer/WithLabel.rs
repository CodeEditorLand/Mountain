//! `Timer::WithLabel`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::Telemetry::Metrics::GlobalRegistry;

pub fn Fn(mut self, Key:&str, Value:&str) -> Struct {
		self.Labels.insert(Key.to_string(), Value.to_string());

		self
	}
