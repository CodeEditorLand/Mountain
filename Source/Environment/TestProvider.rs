// @module TestProvider (Environment)
// @description Implements the `TestProvider` trait for `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, test::TestProvider};
use log::warn;
use serde_json::Value;

use super::MountainEnvironment;

#[async_trait]
impl TestProvider for MountainEnvironment {
	async fn RegisterTestController(&self, _controller_id:String, _label:String) -> Result<(), CommonError> {
		warn!("[TestProvider] RegisterTestController is not implemented.");
		Ok(())
	}

	async fn RunTests(&self, _controller_id:String, _test_run_request:Value) -> Result<(), CommonError> {
		warn!("[TestProvider] RunTests is not implemented.");
		Ok(())
	}
}

impl Requires<Arc<dyn TestProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TestProvider + Send + Sync> { Arc::new(self.clone()) }
}
