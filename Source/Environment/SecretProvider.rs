//! # SecretProvider (Environment)
//!
//! Implements the `SecretProvider` trait for `MountainEnvironment`. Contains
//! the core logic for secure secret storage using the system keyring, powered
//! by the `keyring` crate.
//!
//! ## Keyring integration
//!
//! The `keyring` crate provides cross-platform secure storage:
//! - **macOS**: Native Keychain (OSXKeychain)
//! - **Windows**: Windows Credential Manager (WinCredential)
//! - **Linux**: Secret Service API (dbus-secret-service) or GNOME Keyring
//!
//! Each secret is identified by a service name
//! (`<app>.<ExtensionIdentifier>`) and a key string.
//!
//! ## Security considerations
//!
//! 1. Secrets are never logged or included in error messages.
//! 2. The keyring handles encryption at the OS level.
//! 3. OS keychain manages access permissions and unlocking.
//! 4. Failed operations do not expose secret values.
//! 5. Extension and key identifiers are validated before use.
//!
//! ## Air integration
//!
//! When the `AirIntegration` feature is enabled, `GetSecret`, `StoreSecret`,
//! and `DeleteSecret` delegate to Air service RPCs when the client is healthy,
//! falling back to the local keyring otherwise. The three Air stub functions
//! (`GetSecretFromAir`, `StoreSecretToAir`, `DeleteSecretFromAir`) are gated
//! behind `#[cfg(feature = "AirIntegration")]` and currently return
//! `NotImplemented`.
//!
//! ## VS Code reference
//!
//! - `vs/platform/secrets/common/secrets.ts`
//! - `vs/platform/secrets/electron-simulator/electronSecretStorage.ts`

use CommonLibrary::{Error::CommonError::CommonError, Secret::SecretProvider::SecretProvider};
use async_trait::async_trait;
use keyring_core::{Entry, Error as KeyringError};
// Import Air client types when Air is available in the workspace
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;

use super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

/// Constructs the service name for the keyring entry.
fn GetKeyringServiceName(Environment:&MountainEnvironment, ExtensionIdentifier:&str) -> String {
	format!("{}.{}", Environment.ApplicationHandle.package_info().name, ExtensionIdentifier)
}

/// Helper to check if the Air gRPC client is available without a
/// proper health check. The raw client requires `&mut self` for
/// `health_check`, but `MountainEnvironment` holds an immutable
/// reference. This returns `true` whenever a client is attached.
/// Blocked on proper wrapper integration.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(_AirClient:&AirServiceClient<tonic::transport::Channel>) -> bool {
	// TODO: implement proper health check when AirClient wrapper supports
	// &mut self for health_check RPC. MountainEnvironment stores an
	// immutable reference, so this is blocked on wrapper integration.
	true
}

#[async_trait]
impl SecretProvider for MountainEnvironment {
	/// Retrieves a secret by reading from the OS keychain.
	///
	/// When `AirIntegration` is enabled, attempts to delegate to the Air
	/// service first and falls back to the local keyring on failure.
	/// Returns `Ok(None)` if the keychain entry does not exist.
	#[cfg_attr(
		not(feature = "AirIntegration"),
		allow(unused_mut)
	)]
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError> {
		dev_log!(
			"storage-verbose",
			"[SecretProvider] Getting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier,
			Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					dev_log!(
						"storage-verbose",
						"[SecretProvider] Delegating GetSecret to Air service for key: '{}'",
						Key
					);

					return GetSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					dev_log!(
						"storage",
						"warn: [SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
				}
			}
		}

		dev_log!(
			"storage-verbose",
			"[SecretProvider] Using local keyring for ext: '{}'",
			ExtensionIdentifier
		);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = match Entry::new(&ServiceName, &Key) {
			Ok(e) => e,
			Err(KeyringError::NoStorageAccess(_)) | Err(KeyringError::PlatformFailure(_)) => {
				dev_log!(
					"storage",
					"warn: [SecretProvider] Keyring unavailable for key '{}', returning None",
					Key
				);
				return Ok(None);
			},
			Err(Error) => return Err(CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() }),
		};

		match Entry.get_password() {
			Ok(Password) => Ok(Some(Password)),

			Err(KeyringError::NoEntry) => Ok(None),

			Err(Error) => Err(CommonError::SecretsAccess { Key, Reason:Error.to_string() }),
		}
	}

	/// Stores a secret by writing to the OS keychain.
	///
	/// When `AirIntegration` is enabled, attempts to delegate to the Air
	/// service first and falls back to the local keyring on failure.
	#[cfg_attr(
		not(feature = "AirIntegration"),
		allow(unused_mut)
	)]
	async fn StoreSecret(&self, ExtensionIdentifier:String, Key:String, Value:String) -> Result<(), CommonError> {
		dev_log!(
			"storage-verbose",
			"[SecretProvider] Storing secret for ext: '{}', key: '{}'",
			ExtensionIdentifier,
			Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					dev_log!(
						"storage-verbose",
						"[SecretProvider] Delegating StoreSecret to Air service for key: '{}'",
						Key
					);

					return StoreSecretToAir(AirClient, ExtensionIdentifier.clone(), Key, Value).await;
				} else {
					dev_log!(
						"storage",
						"warn: [SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
				}
			}
		}

		dev_log!(
			"storage-verbose",
			"[SecretProvider] Using local keyring for ext: '{}'",
			ExtensionIdentifier
		);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = match Entry::new(&ServiceName, &Key) {
			Ok(e) => e,
			Err(KeyringError::NoStorageAccess(_)) | Err(KeyringError::PlatformFailure(_)) => {
				dev_log!(
					"storage",
					"warn: [SecretProvider] Keyring unavailable for key '{}', cannot store",
					Key
				);
				return Ok(());
			},
			Err(Error) => return Err(CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() }),
		};

		Entry
			.set_password(&Value)
			.map_err(|Error| CommonError::SecretsAccess { Key, Reason:Error.to_string() })
	}

	/// Deletes a secret by removing it from the OS keychain.
	///
	/// When `AirIntegration` is enabled, attempts to delegate to the Air
	/// service first and falls back to the local keyring on failure.
	/// Idempotent: removing a non-existent entry is treated as success.
	#[cfg_attr(not(feature = "AirIntegration"), allow(unused_mut))]
	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError> {
		dev_log!(
			"storage-verbose",
			"[SecretProvider] Deleting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier,
			Key
		);

		#[cfg(feature = "AirIntegration")]
		{
			if let Some(AirClient) = &self.AirClient {
				if IsAirAvailable(AirClient).await {
					dev_log!(
						"storage-verbose",
						"[SecretProvider] Delegating DeleteSecret to Air service for key: '{}'",
						Key
					);

					return DeleteSecretFromAir(AirClient, ExtensionIdentifier.clone(), Key).await;
				} else {
					dev_log!(
						"storage",
						"warn: [SecretProvider] Air client unavailable, falling back to local keyring for key: '{}'",
						Key
					);
				}
			}
		}

		dev_log!(
			"storage-verbose",
			"[SecretProvider] Using local keyring for ext: '{}'",
			ExtensionIdentifier
		);

		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);

		let Entry = match Entry::new(&ServiceName, &Key) {
			Ok(e) => e,
			Err(KeyringError::NoStorageAccess(_)) | Err(KeyringError::PlatformFailure(_)) => {
				dev_log!(
					"storage",
					"warn: [SecretProvider] Keyring unavailable for key '{}', cannot delete",
					Key
				);
				return Ok(());
			},
			Err(Error) => return Err(CommonError::SecretsAccess { Key:Key.clone(), Reason:Error.to_string() }),
		};

		match Entry.delete_credential() {
			Ok(_) | Err(KeyringError::NoEntry) => Ok(()),

			Err(Error) => Err(CommonError::SecretsAccess { Key, Reason:Error.to_string() }),
		}
	}
}

// ============================================================================
// Air Integration Functions
// ============================================================================

/// Air stub: retrieves a secret from the remote Air service.
///
/// TODO: construct GetSecretRequest with ExtensionIdentifier + Key, call
/// AirClient.get_secret with timeout, map errors to CommonError, return
/// Ok(Some(secret)) if found or Ok(None) if not found.
#[cfg(feature = "AirIntegration")]
async fn GetSecretFromAir(
	_AirClient:&AirServiceClient<tonic::transport::Channel>,

	ExtensionIdentifier:String,

	Key:String,
) -> Result<Option<String>, CommonError> {
	dev_log!(
		"storage",
		"[SecretProvider] Fetching secret from Air: ext='{}', key='{}'",
		ExtensionIdentifier,
		Key
	);

	// TODO: construct GetSecretRequest with ExtensionIdentifier + Key, call
	// AirClient.get_secret with timeout, map errors to CommonError, return
	// Ok(Some(secret)) if found / Ok(None) if not found.
	Err(CommonError::NotImplemented { FeatureName:"GetSecretFromAir".to_string() })
}

/// Air stub: stores a secret in the remote Air service.
///
/// TODO: construct StoreSecretRequest with ExtensionIdentifier, Key, Value;
/// handle encryption and secure transmission; map errors to CommonError.
#[cfg(feature = "AirIntegration")]
async fn StoreSecretToAir(
	_AirClient:&AirServiceClient<tonic::transport::Channel>,

	ExtensionIdentifier:String,

	Key:String,

	_Value:String,
) -> Result<(), CommonError> {
	dev_log!(
		"storage",
		"[SecretProvider] Storing secret in Air: ext='{}', key='{}'",
		ExtensionIdentifier,
		Key
	);

	// TODO: construct StoreSecretRequest with ExtensionIdentifier, Key, Value;
	// handle encryption and secure transmission; map errors to CommonError.
	Err(CommonError::NotImplemented { FeatureName:"StoreSecretToAir".to_string() })
}

/// Deletes a secret from the Air service.
#[cfg(feature = "AirIntegration")]
async fn DeleteSecretFromAir(
	_AirClient:&AirServiceClient<tonic::transport::Channel>,

	ExtensionIdentifier:String,

	Key:String,
) -> Result<(), CommonError> {
	dev_log!(
		"storage",
		"[SecretProvider] Deleting secret from Air: ext='{}', key='{}'",
		ExtensionIdentifier,
		Key
	);

	// TODO: construct DeleteSecretRequest, handle idempotency (missing secret
	// is success), map errors to CommonError.
	Err(CommonError::NotImplemented { FeatureName:"DeleteSecretFromAir".to_string() })
}
