//! # SynchronizationProvider Implementation
//!
//! Implements the `SynchronizationProvider` trait for the
//! `MountainEnvironment`. This is currently a stub implementation.

use Common::{Error::CommonError::CommonError, Synchronization::SynchronizationProvider::SynchronizationProvider};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl SynchronizationProvider for MountainEnvironment {
	async fn PushUserData(&self, _UserData:Value) -> Result<(), CommonError> {
		warn!("[SyncProvider] PushUserData is not implemented.");
		// A real implementation would connect to a settings sync service.
		Ok(())
	}

	async fn PullUserData(&self) -> Result<Value, CommonError> {
		warn!("[SyncProvider] PullUserData is not implemented.");
		Ok(Value::Null)
	}
}
