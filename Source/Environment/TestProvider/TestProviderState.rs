#![allow(non_snake_case)]

//! Aggregate state for the TestProvider: registered controllers and
//! currently active test runs. Held inside `ApplicationState` behind a
//! `tokio::sync::RwLock` for concurrent reads during test runs.

use std::collections::HashMap;

use crate::Environment::TestProvider::{TestControllerState, TestRun};

#[derive(Debug)]
pub struct Struct {
	pub Controllers:HashMap<String, TestControllerState::Struct>,
	pub ActiveRuns:HashMap<String, TestRun::Struct>,
}

impl Struct {
	pub fn new() -> Self { Self { Controllers:HashMap::new(), ActiveRuns:HashMap::new() } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
