//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # SynchronizationProvider Implementation
//!
//! Implements the `SynchronizationProvider` trait for the
//! `MountainEnvironment`. This is currently a stub implementation.

#![allow(non_snake_case, non_camel_case_types)]

use Common::{Error::CommonError::CommonError, Synchronization::SynchronizationProvider::SynchronizationProvider};
use async_trait::async_trait;
use log::warn;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl SynchronizationProvider for MountainEnvironment {
	async fn PushUserData(&self, _UserData:Value) -> Result<(), CommonError> {
		warn!("[SyncProvider] PushUserData is not implemented.");

		// A real implementation would connect to a settings sync service,
		// authenticate, and upload the user data payload.
		Ok(())
	}

	async fn PullUserData(&self) -> Result<Value, CommonError> {
		warn!("[SyncProvider] PullUserData is not implemented.");

		// A real implementation would connect to a settings sync service,
		// authenticate, and download the latest user data snapshot.
		Ok(Value::Null)
	}
}
