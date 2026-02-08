//! # Secret Storage State Management
//!
//! This module provides in-memory state management for secret storage operations.
//! In production, this should be replaced with platform-specific secure storage
//! (Keychain on macOS, Credential Manager on Windows, libsecret on Linux).

use std::{
	collections::HashMap,
	sync::Arc,
};
use parking_lot::RwLock;

/// Secret storage entry
///
/// Each secret is stored with a key, value, and associated extension ID
/// for isolation and security.
#[derive(Clone, Debug)]
pub struct SecretEntry {
	/// Secret key
	pub key: String,

	/// Secret value (encrypted in production)
	pub value: String,

	/// Extension ID that owns this secret
	pub extension_id: String,

	/// Timestamp when secret was created
	pub created_at: chrono::DateTime<chrono::Utc>,

	/// Timestamp when secret was last updated
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl SecretEntry {
	/// Create a new secret entry
	pub fn new(key: String, value: String, extension_id: String) -> Self {
		let now = chrono::Utc::now();
		Self {
			key,
			value,
			extension_id,
			created_at: now,
			updated_at: now,
		}
	}

	/// Update the secret value
	pub fn update_value(&mut self, value: String) {
		self.value = value;
		self.updated_at = chrono::Utc::now();
	}
}

/// Secret storage state manager
///
/// This singleton manages the state for all secret storage operations:
/// - Secret storage registry
/// - Extension-specific secret isolation
/// - Secret lifecycle management
///
/// ## Security Considerations
///
/// **WARNING**: This is an in-memory implementation for development purposes.
/// Production deployment MUST use platform-specific secure storage:
/// - **macOS**: Keychain Services via `security` framework
/// - **Windows**: Credential Manager via Windows API
/// - **Linux**: libsecret via `libsecret-1` bindings
///
/// Never store secrets in plaintext in production!
#[derive(Clone)]
pub struct SecretStorageStateManager {
	/// Registry of secrets (key -> SecretEntry)
	secrets: Arc<RwLock<HashMap<String, SecretEntry>>>,
}

impl SecretStorageStateManager {
	/// Create a new secret storage state manager
	pub fn new() -> Self {
		Self {
			secrets: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	// ==================== Secret Operations ====================

	/// Store a secret
	///
	/// If the secret already exists, it will be updated.
	pub fn store_secret(&self, key: String, value: String, extension_id: String) -> Result<(), String> {
		// Validate inputs
		if key.is_empty() {
			return Err("Secret key cannot be empty".to_string());
		}

		// Check if secret already exists
		let mut secrets = self.secrets.write();
		if let Some(entry) = secrets.get_mut(&key) {
			// Validate ownership
			if entry.extension_id != extension_id {
				return Err(format!(
					"Secret '{}' is owned by extension '{}', cannot be modified by '{}'",
					key, entry.extension_id, extension_id
				));
			}
			// Update existing secret
			entry.update_value(value);
		} else {
			// Create new secret
			let entry = SecretEntry::new(key.clone(), value, extension_id);
			secrets.insert(key, entry);
		}

		Ok(())
	}

	/// Get a secret
	///
	/// Returns the secret value if it exists and the extension is authorized.
	pub fn get_secret(&self, key: &str, extension_id: &str) -> Result<String, String> {
		let secrets = self.secrets.read();

		match secrets.get(key) {
			Some(entry) => {
				if entry.extension_id == extension_id {
					Ok(entry.value.clone())
				} else {
					Err(format!(
						"Extension '{}' is not authorized to access secret '{}'",
						extension_id, key
					))
				}
			},
			None => Err(format!("Secret '{}' not found", key)),
		}
	}

	/// Delete a secret
	///
	/// Returns success if the secret was deleted, error otherwise.
	pub fn delete_secret(&self, key: &str, extension_id: &str) -> Result<(), String> {
		let mut secrets = self.secrets.write();

		match secrets.get(key) {
			Some(entry) => {
				if entry.extension_id != extension_id {
					return Err(format!(
						"Secret '{}' is owned by extension '{}', cannot be deleted by '{}'",
						key, entry.extension_id, extension_id
					));
				}
			},
			None => {
				// Return success even if secret doesn't exist (idempotent)
				return Ok(());
			},
		}

		secrets.remove(key);
		Ok(())
	}

	/// Check if a secret exists
	pub fn secret_exists(&self, key: &str) -> bool {
		let secrets = self.secrets.read();
		secrets.contains_key(key)
	}

	/// List all secret keys for an extension
	pub fn list_secrets_for_extension(&self, extension_id: &str) -> Vec<String> {
		let secrets = self.secrets.read();
		secrets
			.values()
			.filter(|entry| entry.extension_id == extension_id)
			.map(|entry| entry.key.clone())
			.collect()
	}

	/// Delete all secrets for an extension
	///
	/// Returns the number of secrets deleted.
	pub fn delete_secrets_for_extension(&self, extension_id: &str) -> usize {
		let mut secrets = self.secrets.write();
		let initial_count = secrets.len();

		secrets.retain(|_key, entry| entry.extension_id != extension_id);

		initial_count - secrets.len()
	}

	/// Get all secrets (for debugging/testing only)
	///
	/// **WARNING**: Never use this in production!
	pub fn _get_all_secrets(&self) -> Vec<SecretEntry> {
		let secrets = self.secrets.read();
		secrets.values().cloned().collect()
	}
}

impl Default for SecretStorageStateManager {
	fn default() -> Self {
		Self::new()
	}
}
