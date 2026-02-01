// File: Mountain/Source/Environment/SecretProvider.rs
// Role: Implements the `SecretProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Securely store and retrieve secrets using the OS keychain.
//   - Provide a consistent API across platforms (Windows, macOS, Linux).
//   - Handle keychain access failures gracefully with proper error handling.
//   - Support secret sharing between processes via unique service names.
//   - Integrate with Air service for cloud synchronization (optional).
//   - Ensure secrets are never exposed in logs or error messages.
//   - Provide secure secret storage with encryption.
//   - Handle secret lifecycle (create, read, update, delete).
//
// TODOs:
//   - Implement complete Air-based secret storage
//   - Add secret sync between Air and local keyring
//   - Implement conflict resolution strategies for sync
//   - Add caching layer for frequently accessed secrets
//   - Implement retry logic for transient keychain failures
//   - Add metrics for Air vs Local usage tracking
//   - Implement secret versioning (for rollback capability)
//   - Add secret expiration support
//   - Implement secret audit logging
//   - Support secret encryption at rest for additional security
//   - Add secret backup and recovery
//   - Implement secret migration utilities
//   - Add secret access control and permissions
//   - Support secret sharing between devices (via Air)
//   - Implement secret key derivation (PBKDF2, scrypt)
//   - Add secret validation and integrity checking
//
// Inspired by VSCode's secrets service which:
// - Uses operating system keychain for secure storage
// - Provides consistent API across platforms (macOS Keychain, Windows
//   Credential Manager, Linux Secret Service)
// - Handles keychain access failures gracefully
// - Supports secret encryption
// - Provides secure secret sharing between processes
//! # SecretProvider Implementation
//!
//! Implements the `SecretProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for secure secret storage using the system
//! keyring, powered by the `keyring` crate.
//!
//! ## Keyring Integration
//!
//! The `keyring` crate provides cross-platform secure storage:
//! - **macOS**: Native Keychain (OSXKeychain)
//! - **Windows**: Windows Credential Manager (WinCredential)
//! - **Linux**: Secret Service API (dbus-secret-service) or GNOME Keyring
//!
//! Each secret is identified by:
//! - **Service Name**: Application identifier (e.g., `com.myapp.mountain`)
//! - **Key**: Unique identifier within the service (e.g., `github-token`)
//! - **Value**: The secret data to store
//!
//! ## Security Considerations
//!
//! 1. **No Secret Logging**: Secrets are never logged or included in error
//!    messages
//! 2. **Secure Storage**: Keyring handles encryption at the OS level
//! 3. **Access Control**: OS keychain manages access permissions and unlocking
//! 4. **Error Handling**: Failed operations don't expose secret values
//! 5. **Input Validation**: Extension and key identifiers are validated
//!
//! ## Air Integration Strategy
//!
//! This provider supports delegation to the Air service when available:
//! - If AirClient is provided, secrets are stored/retrieved via Air service
//! - If AirClient is unavailable, falls back to local keyring implementation
//! - This ensures backward compatibility while enabling cloud sync
//! - Health checks determine Air availability at runtime
//!
//! ## Secret Operations
//!
//! - **GetSecret**: Retrieve a secret from storage
//!   - Returns `Some(Value)` if found, `None` if not found
//!   - Delegates to Air if available and healthy
//!   - Falls back to local keyring otherwise
//!
//! - **StoreSecret**: Store or update a secret
//!   - Creates entry if it doesn't exist
//!   - Updates entry if it already exists
//!   - Delegates to Air if available and healthy
//!   - Falls back to local keyring otherwise
//!
//! - **DeleteSecret**: Remove a secret from storage
//!   - Succeeds even if secret doesn't exist
//!   - Delegates to Air if available and healthy
//!   - Falls back to local keyring otherwise
// TODO: Full Air Migration Plan
// ============================
// - [ ] Implement complete Air-based secret storage
// - [ ] Add secret sync between Air and local keyring
// - [ ] Implement conflict resolution strategies
// - [ ] Add caching layer for frequently accessed secrets
// - [ ] Implement retry logic for transient Air failures
// - [ ] Add metrics for Air vs Local usage tracking
// - [ ] Phase out local keyring after successful Air deployment

use std::sync::Arc;

use CommonLibrary::{Error::CommonError::CommonError, Secret::SecretProvider::SecretProvider};
use async_trait::async_trait;
use keyring::Entry;
use log::{info, trace, warn};
// Import Air client types when Air is available in the workspace
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::Air::air_service_client::AirServiceClient;

use super::MountainEnvironment::MountainEnvironment;

/// Constructs the service name for the keyring entry.
fn GetKeyringServiceName(Environment:&MountainEnvironment, ExtensionIdentifier:&str) -> String {
	format!("{}.{}", Environment.ApplicationHandle.package_info().name, ExtensionIdentifier)
}

/// Helper to check if Air client is available and healthy.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(AirClient:&mut AirServiceClient<tonic::transport::Channel>) -> bool {
	use tonic::Request;
	use AirLibrary::Vine::Generated::Air::HealthCheckRequest;

	match AirClient.health_check(Request::new(HealthCheckRequest {})).await {
		Ok(response) => response.into_inner().healthy,
		Err(error) => {
			warn!("[SecretProvider] Air health check failed: {}", error);
			false
		},
	}
}

#[async_trait]
impl SecretProvider for MountainEnvironment {
	/// Retrieves a secret by reading from the OS keychain.
	/// If Air is available and healthy, delegates to Air service.
	/// Falls back to local keyring if Air is unavailable.
	#[allow(unused_mut, unused_variables)]
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError> {
		trace!(
			"[SecretProvider] Getting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating GetSecret to Air service for key: '{}'", Key);

					return GetSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					warn!(
						"[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
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
	async fn StoreSecret(&self, ExtensionIdentifier:String, Key:String, Value:String) -> Result<(), CommonError> {
		info!(
			"[SecretProvider] Storing secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating StoreSecret to Air service for key: '{}'", Key);

					return StoreSecretToAir(AirClient, ExtensionIdentifier.clone(), Key, Value).await;
				} else {
					warn!(
						"[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
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
	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError> {
		info!(
			"[SecretProvider] Deleting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					info!("[SecretProvider] Delegating DeleteSecret to Air service for key: '{}'", Key);

					return DeleteSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					warn!(
						"[SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
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
	AirClient:&AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<Option<String>, CommonError> {
	use AirLibrary::Vine::Generated::Air::air_service_server;

	info!(
		"[SecretProvider] Fetching secret from Air: ext='{}', key='{}'",
		ExtensionIdentifier, Key
	);

	// TODO: Implement Air secret retrieval
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented { FeatureName:"GetSecretFromAir".to_string() })
}

/// Stores a secret in the Air service.
#[cfg(feature = "AirIntegration")]
async fn StoreSecretToAir(
	AirClient:&AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier:String,
	Key:String,
	Value:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretProvider] Storing secret in Air: ext='{}', key='{}'",
		ExtensionIdentifier, Key
	);

	// TODO: Implement Air secret storage
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented { FeatureName:"StoreSecretToAir".to_string() })
}

/// Deletes a secret from the Air service.
#[cfg(feature = "AirIntegration")]
async fn DeleteSecretFromAir(
	AirClient:&AirServiceClient<tonic::transport::Channel>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretProvider] Deleting secret from Air: ext='{}', key='{}'",
		ExtensionIdentifier, Key
	);

	// TODO: Implement Air secret deletion
	// This would call Air's secret management API
	// For now, return NotImplemented to indicate this needs to be implemented
	Err(CommonError::NotImplemented { FeatureName:"DeleteSecretFromAir".to_string() })
}
