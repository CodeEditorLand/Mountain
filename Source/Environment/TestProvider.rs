//! # TestProvider Implementation
//!
//! Implements the `TestController` trait for the `MountainEnvironment`. This is
//! currently a stub implementation.

#![allow(non_snake_case, non_camel_case_types)]

use Common::{Error::CommonError::CommonError, Testing::TestController::TestController};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl TestController for MountainEnvironment {
	async fn RegisterTestController(&self, ControllerId:String, Label:String) -> Result<(), CommonError> {
		warn!(
			"[TestProvider] RegisterTestController for '{}' ('{}') is not implemented.",
			ControllerId, Label
		);

		// A full implementation would store the test controller's information
		// in ApplicationState, keyed by its ID and associated with the sidecar
		// that registered it.
		Ok(())
	}

	async fn RunTests(&self, ControllerId:String, TestRunRequest:Value) -> Result<(), CommonError> {
		warn!(
			"[TestProvider] RunTests for '{}' ({:?}) is not implemented.",
			ControllerId, TestRunRequest
		);

		// A full implementation would:
		// 1. Find the sidecar associated with `ControllerId`.
		// 2. Send an RPC request to that sidecar to start the test run.
		// 3. The sidecar would then send back events for test progress, (e.g., test
		//    started, passed, failed), which this provider would handle and forward to
		//    the UI.
		Ok(())
	}
}
