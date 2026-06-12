//! Aggregate state for the TestProvider: registered controllers and
//! currently active test runs. Held inside `ApplicationState` behind a
//! `tokio::sync::RwLock` for concurrent reads during test runs.

use std::collections::HashMap;

use crate::Environment::TestProvider::{TestControllerState, TestRun};

/// Aggregate state for the TestProvider.
///
/// Holds the registry of test controllers and currently active test runs.
/// Stored inside `ApplicationState` behind a `tokio::sync::RwLock`.
#[derive(Debug)]
pub struct Struct {
	/// Registered test controllers, keyed by `ControllerIdentifier`.
	pub Controllers:HashMap<String, TestControllerState::Struct>,
	/// Active test runs, keyed by `RunIdentifier`.
	pub ActiveRuns:HashMap<String, TestRun::Struct>,
}

impl Struct {
	/// Creates a new empty `TestProviderState`.
	pub fn new() -> Self { Self { Controllers:HashMap::new(), ActiveRuns:HashMap::new() } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
