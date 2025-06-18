// @module ScmProvider (Environment)
// @description Implements the `ScmProvider` trait for `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	scm::{ScmProvider, DTO::*},
};
use log::warn;

use super::MountainEnvironment;

#[async_trait]
impl ScmProvider for MountainEnvironment {
	async fn RegisterScmProvider(&self, _provider_data:ScmProviderDTO) -> Result<u32, CommonError> {
		warn!("[ScmProvider] RegisterScmProvider is not implemented.");
		// A real implementation would store the provider and return a handle.
		Ok(1)
	}

	async fn UpdateScmGroup(&self, _provider_handle:u32, _group_data:ScmGroupDTO) -> Result<(), CommonError> {
		warn!("[ScmProvider] UpdateScmGroup is not implemented.");
		// A real implementation would find the provider and send an update event to the
		// User Interface.
		Ok(())
	}
}

impl Requires<Arc<dyn ScmProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ScmProvider + Send + Sync> { Arc::new(self.clone()) }
}
