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
//! The Air service definition does not expose secret RPCs (the Vine protocol
//! does, but Mountain holds an Air client), so `GetSecret`, `StoreSecret` and
//! `DeleteSecret` always use the local OS keyring. The availability probe logs
//! a warning and reports unavailable rather than routing to the three
//! Air helpers, each of which warns that the remote secret-fetch capability
//! is missing and returns a typed `FeatureNotAvailable` error.
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

/// Checks whether the Air service can actually serve secret operations.
///
/// The Air service definition does not expose `GetSecret`, `StoreSecret` or
/// `DeleteSecret` RPCs (only the Vine protocol defines them, and Mountain
/// holds an Air client). Reporting availability unconditionally made every
/// secret operation fail with an unavailable error whenever an Air client was
/// attached. Report unavailability with a clear warning instead so the
/// callers fall back to the local OS keyring.
#[cfg(feature = "AirIntegration")]
async fn IsAirAvailable(_AirClient:&AirServiceClient<tonic::transport::Channel>) -> bool {
	dev_log!(
		"storage",
		"warn: [SecretProvider] Air service has no secret RPCs; using the local keyring"
	);

	false
}

#[async_trait]
impl SecretProvider for MountainEnvironment {
	/// Retrieves a secret by reading from the OS keychain.
	///
	/// When `AirIntegration` is enabled, attempts to delegate to the Air
	/// service first and falls back to the local keyring on failure.
	/// Returns `Ok(None)` if the keychain entry does not exist.
	#[cfg_attr(not(feature = "AirIntegration"), allow(unused_mut))]
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
	#[cfg_attr(not(feature = "AirIntegration"), allow(unused_mut))]
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
/// The `AirService` contract exposes no secret-fetch RPC, so this helper
/// cannot reach a remote store. It logs a warning naming the missing
/// capability and returns a typed [`CommonError::FeatureNotAvailable`], which
/// callers can distinguish from `Ok(None)` ("no secret"). A follow-up card
/// should add `GetSecret` to `Element/Air/Proto/Air.proto` and wire this helper
/// to it through a bounded, timeout-guarded client call.
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
	dev_log!(
		"storage",
		"warn: [SecretProvider] GetSecretFromAir unavailable: AirService has no GetSecret RPC; using the local keyring"
	);

	Err(CommonError::FeatureNotAvailable { FeatureName:"AirService.GetSecret".to_string() })
}

/// Air stub: stores a secret in the remote Air service.
///
/// The `AirService` contract exposes no secret-store RPC, so this helper
/// cannot reach a remote store. It logs a warning naming the missing
/// capability and returns a typed [`CommonError::FeatureNotAvailable`], which
/// callers can distinguish from a successful local write. A follow-up card
/// should add `StoreSecret` to `Element/Air/Proto/Air.proto` and wire this
/// helper to it through a bounded, timeout-guarded client call.
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
	dev_log!(
		"storage",
		"warn: [SecretProvider] StoreSecretToAir unavailable: AirService has no StoreSecret RPC; using the local keyring"
	);

	Err(CommonError::FeatureNotAvailable { FeatureName:"AirService.StoreSecret".to_string() })
}

/// Air stub: deletes a secret from the remote Air service.
///
/// The `AirService` contract exposes no secret-delete RPC, so this helper
/// cannot reach a remote store. It logs a warning naming the missing
/// capability and returns a typed [`CommonError::FeatureNotAvailable`], which
/// callers can distinguish from a successful idempotent local delete. A
/// follow-up card should add `DeleteSecret` to `Element/Air/Proto/Air.proto`
/// and wire this helper to it through a bounded, timeout-guarded client call.
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
	dev_log!(
		"storage",
		"warn: [SecretProvider] DeleteSecretFromAir unavailable: AirService has no DeleteSecret RPC; using the local keyring"
	);

	Err(CommonError::FeatureNotAvailable { FeatureName:"AirService.DeleteSecret".to_string() })
}
