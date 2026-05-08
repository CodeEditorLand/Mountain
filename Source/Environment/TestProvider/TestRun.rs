#![allow(non_snake_case)]

//! Active test run record: run identifier, owning controller, current
//! status, start instant, and per-test results keyed by test
//! identifier. Stored in `TestProviderState::Struct::ActiveRuns` for
//! the duration of the run.

use std::collections::HashMap;

use crate::Environment::TestProvider::{TestResult, TestRunStatus};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Struct {
	pub RunIdentifier:String,

	pub ControllerIdentifier:String,

	pub Status:TestRunStatus::Enum,

	pub StartedAt:std::time::Instant,

	pub Results:HashMap<String, TestResult::Struct>,
}
