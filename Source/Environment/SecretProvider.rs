//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # SecretProvider Implementation
//!
//! Implements the `SecretProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for secure secret storage using the system
//! keyring, powered by the `keyring` crate.
//!
//! ## Air Integration Strategy
//!
//! This provider supports delegation to the Air service when available:
//! - If AirClient is provided, secrets are stored/retrieved via Air service
//! - If AirClient is unavailable, falls back to local keyring implementation
//! - This ensures backward compatibility while enabling cloud sync
//!
//! TODO: Full Air Migration Plan
//! ============================
//! - [ ] Implement complete Air-based secret storage
//! - [ ] Add secret sync between Air and local keyring
//! - [ ] Implement conflict resolution strategies
//! - [ ] Add caching layer for frequently accessed secrets
//! - [ ] Implement retry logic for transient Air failures
//! - [ ] Add metrics for Air vs Local usage tracking
//! - [ ] Phase out local keyring after successful Air deployment

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Error::CommonError::CommonError, Secret::SecretProvider::SecretProvider};
use async_trait::async_trait;
use keyring::Entry;
use log::{info, trace, warn};

use super::MountainEnvironment::MountainEnvironment;

// Import Air client types when Air is available in the workspace
#[cfg(feature = "AirIntegration")]
use Air::Vine::Generated::air_service_client::AirServiceClient;
#[cfg(feature = "AirIntegration")]
use Air::Vine::Generated::air_service_client::air_service_server;

/// Constructs the service name for the keyring entry.
fn GetKeyringServiceName(Environment: &MountainEnvironment, ExtensionIdentifier: &str) -> String {
	format!("{}.{}", Environment.ApplicationHandle.package_info().name, ExtensionIdentifier)
}

/// Helper to check if Air client is available and healthy.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(AirClient: &AirServiceClient<tonic::transport::Channel>) -> bool {
	use tonic::Request;

	match AirClient.health_check(Request::new(air_service_server::HealthCheckRequest {})).await {
		Ok(response) => response.into_inner().healthy,
		Err(error) => {
			warn!("[SecretProvider] Air health check failed: {}", error);
			false
		}
	}
}

#[async_trait]
impl SecretProvider for MountainEnvironment {
	/// Retrieves a secret by reading from the OS keychain.
	/// If Air is available and healthy, delegates to Air service.
	/// Falls back to local keyring if Air is unavailable.
	#[allow(unused_mut, unused_variables)]
	async fn GetSecret(
		&self,
		ExtensionIdentifier: String,
		Key: String,
	) -> Result<Option<String>, CommonError> {
		trace!("[SecretProvider] Getting secret for ext: '{}', key: '{}'", ExtensionIdentifier, Key);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating GetSecret to Air service for key: '{}'", Key);

					return GetSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					warn!("[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'", Key);
				}
			}
		}

		info!("[SecretProvider] Using local keyring for ext: '{}'", ExtensionIdentifier);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|Error| CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() })?;

		match Entry.get_password() {
			Ok(Password) => Ok(Some(Password)),

			Err(keyring::Error::NoEntry) => Ok(None),

			Err(Error) => Err(CommonError::SecretsAccess { Key, Reason:Error.to_string() }),
		}
	}

	/// Stores a secret by writing to the OS keychain.
	/// If Air is available and healthy, delegates to Air service.
	/// Falls back to local keyring if Air is unavailable.
	#[allow(unused_mut, unused_variables)]
	async fn StoreSecret(
		&self,
		ExtensionIdentifier: String,
		Key: String,
		Value: String,
	) -> Result<(), CommonError> {
		info!("[SecretProvider] Storing secret for ext: '{}', key: '{}'", ExtensionIdentifier, Key);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating StoreSecret to Air service for key: '{}'", Key);

					return StoreSecretToAir(AirClient, ExtensionIdentifier.clone(), Key, Value).await;
				} else {
					warn!("[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'", Key);
				}
			}
		}

		info!("[SecretProvider] Using local keyring for ext: '{}'", ExtensionIdentifier);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|Error| CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() })?;

		Entry
			.set_password(&Value)
			.map_err(|Error| CommonError::SecretsAccess { Key, Reason:Error.to_string() })
	}

	/// Deletes a secret by removing it from the OS keychain.
	/// If Air is available and healthy, delegates to Air service.
	/// Falls back to local keyring if Air is unavailable.
	#[allow(unused_mut, unused_variables)]
	async fn DeleteSecret(
		&self,
		ExtensionIdentifier: String,
		Key: String,
	) -> Result<(), CommonError> {
		info!("[SecretProvider] Deleting secret for ext: '{}', key: '{}'", ExtensionIdentifier, Key);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating DeleteSecret to Air service for key: '{}'", Key);

					return DeleteSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					warn!("[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'", Key);
				}
			}
		}

		info!("[SecretProvider] Using local keyring for ext: '{}'", ExtensionIdentifier);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|Error| CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() })?;

		match Entry.delete_credential() {
			Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),

			Err(Error) => Err(CommonError::SecretsAccess { Key, Reason:Error.to_string() }),
		}
	}
}

// ============================================================================
// Air Integration Functions
// ============================================================================

#[cfg(feature = "AirIntegration")]
use tonic::Request;

/// Retrieves a secret from the Air service.
#[cfg(feature = "AirIntegration")]
async fn GetSecretFromAir(
	AirClient: &AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier: String,
	Key: String,
) -> Result<Option<String>, CommonError> {
	use Air::Vine::Generated::air_service_client::air_service_server;

	info!("[SecretProvider] Fetching secret from Air: ext='{}', key='{}'", ExtensionIdentifier, Key);

	// TODO: Implement Air secret retrieval
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented {
		FeatureName: "GetSecretFromAir".to_string(),
	})
}

/// Stores a secret in the Air service.
#[cfg(feature = "AirIntegration")]
async fn StoreSecretToAir(
	AirClient: &AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier: String,
	Key: String,
	Value: String,
) -> Result<(), CommonError> {
	info!("[SecretProvider] Storing secret in Air: ext='{}', key='{}'", ExtensionIdentifier, Key);

	// TODO: Implement Air secret storage
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented {
		FeatureName: "StoreSecretToAir".to_string(),
	})
}

/// Deletes a secret from the Air service.
#[cfg(feature = "AirIntegration")]
async fn DeleteSecretFromAir(
	AirClient: &AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier: String,
	Key: String,
) -> Result<(), CommonError> {
	info!("[SecretProvider] Deleting secret from Air: ext='{}', key='{}'", ExtensionIdentifier, Key);

	// TODO: Implement Air secret deletion
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented {
		FeatureName: "DeleteSecretFromAir".to_string(),
	})
}
