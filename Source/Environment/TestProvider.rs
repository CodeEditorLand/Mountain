//! # TestProvider Implementation
//!
//! Implements the `TestController` trait for the `MountainEnvironment`. This is
//! currently a stub implementation.

use Common::{Error::CommonError, Testing::TestController};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;

use super::MountainEnvironment;

#[async_trait]
impl TestController for MountainEnvironment {
	async fn RegisterTestController(&self, _ControllerId:String, _Label:String) -> Result<(), CommonError> {
		warn!("[TestProvider] RegisterTestController is not implemented.");
		Ok(())
	}

	async fn RunTests(&self, _ControllerId:String, _TestRunRequest:Value) -> Result<(), CommonError> {
		warn!("[TestProvider] RunTests is not implemented.");
		Ok(())
	}
}
