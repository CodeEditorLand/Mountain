//! # SecretStorageService Implementation
//!
//! This module implements secret storage-related gRPC service methods for the
//! Mountain backend. These methods handle secure storage and retrieval of
//! sensitive data such as API keys, tokens, and credentials.
//!
//! ## Service Responsibilities
//!
//! - **Get Secret**: Retrieve a secret from storage
//! - **Store Secret**: Store a secret in storage
//! - **Delete Secret**: Delete a secret from storage
//!
//! ## Architecture
//!
//! The SecretStorageService maintains references to:
//! - `MountainEnvironment`: Access to all Mountain services
//! - Secure storage backend for encrypted secret persistence
//!
//! ## Implementation Notes
//!
//! This service is a subset of the main CocoonService, focusing specifically
//! on secret storage operations. It provides a secure interface for
//! extensions to store sensitive data that should not be exposed to the user.

use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, info, warn};
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
use CommonLibrary::Environment::Requires::Requires;

// Import generated protobuf types
use crate::Vine::Generated::{
	// Common types
	Empty,

	// Secret Storage
	GetSecretRequest,
	GetSecretResponse,
	StoreSecretRequest,
	DeleteSecretRequest,
};

// Import state management
use super::SecretStorageState::SecretStorageStateManager;

/// SecretStorageService handles secure secret storage operations
///
/// This service manages:
/// - Retrieval of stored secrets
/// - Secure storage of new secrets
/// - Deletion of secrets when no longer needed
///
/// ## Security Considerations
///
/// - Secrets are encrypted at rest (in production)
/// - Secrets are never logged
/// - Access to secrets is restricted to authorized extensions
///
/// **NOTE**: This implementation uses in-memory storage for development.
/// In production, it should use platform-specific secure storage:
/// - macOS: Keychain Services
/// - Windows: Credential Manager
/// - Linux: libsecret
#[derive(Clone)]
pub struct SecretStorageService {
	/// Mountain environment providing access to all services
	environment: Arc<MountainEnvironment>,

	/// Secret storage state manager
	state_manager: Arc<SecretStorageStateManager>,
}

impl SecretStorageService {
	/// Creates a new instance of the SecretStorageService
	///
	/// # Parameters
	/// - `environment`: Mountain environment with access to all services
	///
	/// # Returns
	/// A new SecretStorageService instance
	pub fn new(environment: Arc<MountainEnvironment>) -> Self {
		info!("[SecretStorageService] New instance created");

		Self {
			environment,
			state_manager: Arc::new(SecretStorageStateManager::new()),
		}
	}
}

impl SecretStorageService {
	// ==================== Secret Storage Operations ====================

	/// Retrieve a secret from storage
	///
	/// # Parameters
	/// - `key`: The key identifying the secret
	///
	/// # Returns
	/// The secret value, or an error if not found
	///
	/// # Errors
	/// - `NOT_FOUND`: The secret does not exist
	/// - `PERMISSION_DENIED`: The extension is not authorized to access this secret
	/// - `INTERNAL`: An error occurred while retrieving the secret
	pub async fn get_secret_impl(&self, key: &str, extension_id: &str) -> Result<String, Status> {
		debug!("[SecretStorageService] Getting secret for key: {}", key);

		// Use in-memory state manager for development
		// In production, this should use platform-specific secure storage
		match self.state_manager.get_secret(key, extension_id) {
			Ok(value) => {
				debug!("[SecretStorageService] Secret retrieved successfully for key: {}", key);
				Ok(value)
			},
			Err(error) => {
				debug!("[SecretStorageService] Failed to retrieve secret: {}", error);
				Err(Status::not_found(error))
			},
		}
	}

	/// Store a secret in storage
	///
	/// # Parameters
	/// - `key`: The key to store the secret under
	/// - `value`: The secret value to store
	///
	/// # Returns
	/// Success status
	///
	/// # Errors
	/// - `INVALID_ARGUMENT`: Key or value is invalid
	/// - `INTERNAL`: An error occurred while storing the secret
	pub async fn store_secret_impl(
		&self,
		key: &str,
		value: &str,
		extension_id: &str,
	) -> Result<(), Status> {
		debug!("[SecretStorageService] Storing secret for key: {}", key);

		// Use in-memory state manager for development
		// In production, this should use platform-specific secure storage
		match self.state_manager.store_secret(key.to_string(), value.to_string(), extension_id.to_string()) {
			Ok(_) => {
				info!(
					"[SecretStorageService] Secret stored successfully for key: {}",
					key
				);
				Ok(())
			},
			Err(err) => {
				error!("[SecretStorageService] Failed to store secret: {}", err);
				Err(Status::internal(err))
			},
		}
	}

	/// Delete a secret from storage
	///
	/// # Parameters
	/// - `key`: The key identifying the secret to delete
	///
	/// # Returns
	/// Success status
	///
	/// # Errors
	/// - `NOT_FOUND`: The secret does not exist
	/// - `INTERNAL`: An error occurred while deleting the secret
	pub async fn delete_secret_impl(&self, key: &str, extension_id: &str) -> Result<(), Status> {
		debug!("[SecretStorageService] Deleting secret for key: {}", key);

		// Use in-memory state manager for development
		// In production, this should use platform-specific secure storage
		match self.state_manager.delete_secret(key, extension_id) {
			Ok(_) => {
				info!(
					"[SecretStorageService] Secret deleted successfully for key: {}",
					key
				);
				Ok(())
			},
			Err(err) => {
				error!("[SecretStorageService] Failed to delete secret: {}", err);
				Err(Status::internal(err))
			},
		}
	}

	/// Check if a secret exists
	///
	/// # Parameters
	/// - `key`: The key to check
	///
	/// # Returns
	/// True if the secret exists, false otherwise
	pub async fn secret_exists(&self, key: &str) -> bool {
		self.state_manager.secret_exists(key)
	}

	/// List all secret keys for an extension
	///
	/// # Parameters
	/// - `extension_id`: The extension ID to list secrets for
	///
	/// # Returns
	/// Vector of secret keys owned by the extension
	pub async fn list_secrets_for_extension(
		&self,
		extension_id: &str,
	) -> Vec<String> {
		debug!(
			"[SecretStorageService] Listing secrets for extension: {}",
			extension_id
		);

		self.state_manager.list_secrets_for_extension(extension_id)
	}

	/// Delete all secrets for an extension
	///
	/// This is called when an extension is uninstalled or disabled.
	///
	/// # Parameters
	/// - `extension_id`: The extension ID to delete secrets for
	///
	/// # Returns
	/// Number of secrets deleted
	pub async fn delete_secrets_for_extension(
		&self,
		extension_id: &str,
	) -> usize {
		info!(
			"[SecretStorageService] Deleting all secrets for extension: {}",
			extension_id
		);

		self.state_manager.delete_secrets_for_extension(extension_id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// TODO: Add unit tests for SecretStorageService methods
	// These tests should verify:
	// - Secret storage and retrieval
	// - Secret deletion
	// - Extension-specific secret isolation
	// - Error handling for missing secrets
	//
	// Note: Tests should use a mock secure storage backend to avoid
	// requiring actual platform-specific secure storage during testing.
}
