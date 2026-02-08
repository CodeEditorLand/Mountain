//! # CommandService Implementation
//!
//! This module implements command-related gRPC service methods for the
//! Mountain backend. These methods handle registration, execution, and
//! unregistration of extension commands.
//!
//! ## Service Responsibilities
//!
//! - **Command Registration**: Register new commands from extensions
//! - **Command Execution**: Execute commands on behalf of extensions
//! - **Command Unregistration**: Remove previously registered commands
//!
//! ## Architecture
//!
//! The CommandService maintains references to:
//! - `MountainEnvironment`: Access to all Mountain services
//! - Command registry for storing command metadata
//! - CommandExecutor for executing commands
//!
//! ## Implementation Notes
//!
//! This service is a subset of the main CocoonService, focusing specifically
//! on command operations. It integrates with the existing Command/Bootstrap
//! module for native command execution.

use std::{
	collections::HashMap,
	sync::Arc,
};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
use CommonLibrary::Environment::Requires::Requires;

// Import generated protobuf types
use crate::Vine::Generated::{
	// Common types
	Empty,
	Argument,

	// Commands
	RegisterCommandRequest,
	ExecuteCommandRequest,
	ExecuteCommandResponse,
	UnregisterCommandRequest,
};

/// Command metadata stored in the registry
#[derive(Clone, Debug)]
struct CommandMetadata {
	/// Unique command identifier
	command_id: String,

	/// Extension that owns this command
	extension_id: String,

	/// Display title for the command
	title: String,

	/// Registration timestamp
	registered_at: chrono::DateTime<chrono::Utc>,
}

/// CommandService handles command registration and execution
///
/// This service manages:
/// - Registering commands from extensions
/// - Executing commands via the CommandExecutor
/// - Unregistering commands when extensions are disabled
#[derive(Clone)]
pub struct CommandService {
	/// Mountain environment providing access to all services
	environment: Arc<MountainEnvironment>,

	/// Registry of registered commands
	/// Maps command_id to command metadata
	commands: Arc<parking_lot::RwLock<HashMap<String, CommandMetadata>>>,
}

impl CommandService {
	/// Creates a new instance of the CommandService
	///
	/// # Parameters
	/// - `environment`: Mountain environment with access to all services
	///
	/// # Returns
	/// A new CommandService instance
	pub fn new(environment: Arc<MountainEnvironment>) -> Self {
		info!("[CommandService] New instance created");

		Self {
			environment,
			commands: Arc::new(parking_lot::RwLock::new(HashMap::new())),
		}
	}
}

impl CommandService {
	// ==================== Command Registration ====================

	/// Register a new command
	///
	/// # Parameters
	/// - `command_id`: Unique identifier for the command
	/// - `extension_id`: Extension that owns the command
	/// - `title`: Display title for the command
	///
	/// # Returns
	/// Success status
	pub async fn register_command_impl(
		&self,
		command_id: &str,
		extension_id: &str,
		title: &str,
	) -> Result<(), Status> {
		info!(
			"[CommandService] Registering command '{}' from extension '{}'",
			command_id, extension_id
		);

		// Check if command is already registered
		{
			let commands = self.commands.read();
			if commands.contains_key(command_id) {
				warn!("[CommandService] Command '{}' already registered", command_id);
				return Err(Status::already_exists(format!(
					"Command '{}' is already registered",
					command_id
				)));
			}
		}

		// Create command metadata
		let metadata = CommandMetadata {
			command_id: command_id.to_string(),
			extension_id: extension_id.to_string(),
			title: title.to_string(),
			registered_at: chrono::Utc::now(),
		};

		// Store in registry
		{
			let mut commands = self.commands.write();
			commands.insert(command_id.to_string(), metadata);
		}

		debug!(
			"[CommandService] Command '{}' registered successfully",
			command_id
		);

		// Register with CommandExecutor from MountainEnvironment
		let command_executor = self.environment.Require();

		match command_executor
			.RegisterCommand(extension_id.to_string(), command_id.to_string())
			.await
		{
			Ok(_) => {
				debug!("[CommandService] Command '{}' registered successfully", command_id);
				Ok(())
			},
			Err(err) => {
				error!("[CommandService] Failed to register command '{}': {}", command_id, err);
				// Clean up from local registry
				let mut commands = self.commands.write();
				commands.remove(command_id);
				Err(Status::internal(format!("Failed to register command: {}", err)))
			},
		}
	}

	/// Execute a command
	///
	/// # Parameters
	/// - `command_id`: The command to execute
	/// - `arguments`: Command arguments (optional)
	///
	/// # Returns
	/// Command execution result or error
	pub async fn execute_command_impl(
		&self,
		command_id: &str,
		arguments: &[Argument],
	) -> Result<Vec<u8>, Status> {
		debug!(
			"[CommandService] Executing command '{}' with {} arguments",
			command_id,
			arguments.len()
		);

		// Check if command is registered
		{
			let commands = self.commands.read();
			if !commands.contains_key(command_id) {
				warn!("[CommandService] Command '{}' not found", command_id);
				return Err(Status::not_found(format!(
					"Command '{}' is not registered",
					command_id
				)));
			}
		}

		// Use CommandExecutor from MountainEnvironment
		let command_executor = self.environment.Require();

		// Convert Argument protobuf types to serde_json::Value
		let argument_value: serde_json::Value = if arguments.is_empty() {
			serde_json::json!({})
		} else {
			serde_json::to_value(arguments).unwrap_or(serde_json::json!({}))
		};

		match command_executor
			.ExecuteCommand(command_id.to_string(), argument_value)
			.await
		{
			Ok(result) => {
				debug!("[CommandService] Command '{}' executed successfully", command_id);
				// Serialize result to bytes
				match serde_json::to_vec(&result) {
					Ok(bytes) => Ok(bytes),
					Err(err) => {
						error!("[CommandService] Failed to serialize command result: {}", err);
						Err(Status::internal("Failed to serialize command result"))
					},
				}
			},
			Err(err) => {
				error!("[CommandService] Failed to execute command '{}': {}", command_id, err);
				Err(Status::internal(format!("Failed to execute command: {}", err)))
			},
		}
	}

	/// Unregister a command
	///
	/// # Parameters
	/// - `command_id`: The command to unregister
	///
	/// # Returns
	/// Success status
	pub async fn unregister_command_impl(&self, command_id: &str) -> Result<(), Status> {
		info!("[CommandService] Unregistering command '{}'", command_id);

		// Remove from registry
		let removed = {
			let mut commands = self.commands.write();
			commands.remove(command_id).is_some()
		};

		if !removed {
			warn!("[CommandService] Command '{}' was not registered", command_id);
			return Err(Status::not_found(format!(
				"Command '{}' is not registered",
				command_id
			)));
		}

		debug!(
			"[CommandService] Command '{}' unregistered successfully",
			command_id
		);

		// Get extension ID from command metadata before removal
		let extension_id = {
			let commands = self.commands.read();
			commands.get(command_id).map(|cmd| cmd.extension_id.clone())
		};

		if let Some(ext_id) = extension_id {
			// Unregister from CommandExecutor
			let command_executor = self.environment.Require();
			if let Err(err) = command_executor.UnregisterCommand(ext_id, command_id.to_string()).await {
				warn!("[CommandService] Failed to unregister command from executor: {}", err);
			}
		}

		Ok(())
	}

	/// Get all registered commands
	///
	/// # Returns
	/// Vector of command metadata
	pub fn get_all_commands(&self) -> Vec<CommandMetadata> {
		let commands = self.commands.read();
		commands.values().cloned().collect()
	}

	/// Get commands for a specific extension
	///
	/// # Parameters
	/// - `extension_id`: The extension ID to filter by
	///
	/// # Returns
	/// Vector of command metadata for the extension
	pub fn get_commands_for_extension(&self, extension_id: &str) -> Vec<CommandMetadata> {
		let commands = self.commands.read();
		commands
			.values()
			.filter(|cmd| cmd.extension_id == extension_id)
			.cloned()
			.collect()
	}

	/// Check if a command is registered
	///
	/// # Parameters
	/// - `command_id`: The command ID to check
	///
	/// # Returns
	/// True if the command is registered, false otherwise
	pub fn is_command_registered(&self, command_id: &str) -> bool {
		let commands = self.commands.read();
		commands.contains_key(command_id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// TODO: Add unit tests for CommandService methods
	// These tests should verify:
	// - Command registration
	// - Duplicate registration prevention
	// - Command execution
	// - Command unregistration
	// - Extension-specific command retrieval
}
