//! # TerminalStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for integrated terminal state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track terminal lifecycle and configuration
//! - Contains runtime handles for PTY I/O
//!
//! # FIELDS
//! - Identifier: Unique terminal identifier
//! - Name: Terminal display name
//! - OSProcessIdentifier: OS process ID
//! - ShellPath: Shell executable path
//! - ShellArguments: Shell launch arguments
//! - CurrentWorkingDirectory: Working directory path
//! - EnvironmentVariables: Environment variable map
//! - IsPTY: PTY mode flag
//! - PTYInputTransmitter: PTY input channel sender
//! - ReaderTaskHandle: Output reader task handle
//! - ProcessWaitHandle: Process wait task handle
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
	sync::{Mutex as TokioMutex, mpsc as TokioMPSC},
	task::JoinHandle,
};

/// Maximum terminal name length
const MAX_TERMINAL_NAME_LENGTH:usize = 128;

/// Maximum shell path length
const MAX_SHELL_PATH_LENGTH:usize = 1024;

/// Maximum number of shell arguments
const MAX_SHELL_ARGUMENTS:usize = 100;

/// Maximum argument string length
const MAX_ARGUMENT_LENGTH:usize = 4096;

/// Maximum number of environment variables
const MAX_ENV_VARS:usize = 1000;

/// Holds the complete state and runtime resources for a single pseudo-terminal
/// (PTY) instance. This includes configuration, process identifiers, and
/// handles for I/O tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStateDTO {
	// --- Identifiers ---
	/// Unique terminal identifier
	pub Identifier:u64,

	/// Terminal display name
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Name:String,

	/// OS process identifier (if running)
	pub OSProcessIdentifier:Option<u32>,

	// --- Configuration ---
	/// Shell executable path
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ShellPath:String,

	/// Shell launch arguments
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub ShellArguments:Vec<String>,

	/// Current working directory
	pub CurrentWorkingDirectory:Option<PathBuf>,

	/// Environment variables map
	#[serde(skip_serializing_if = "Option::is_none")]
	pub EnvironmentVariables:Option<HashMap<String, Option<String>>>,

	/// Whether this is a PTY terminal
	pub IsPTY:bool,

	// --- Runtime Handles ---
	/// Channel for sending input to PTY
	#[serde(skip)]
	pub PTYInputTransmitter:Option<TokioMPSC::Sender<String>>,

	/// Handle for output reader task
	#[serde(skip)]
	pub ReaderTaskHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,

	/// Handle for process wait task
	#[serde(skip)]
	pub ProcessWaitHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
}

impl TerminalStateDTO {
	/// Creates a new `TerminalStateDTO` by parsing terminal options from a
	/// `serde_json::Value` with validation.
	///
	/// # Arguments
	/// * `Identifier` - Unique terminal identifier
	/// * `Name` - Terminal display name
	/// * `OptionsValue` - Terminal options JSON
	/// * `DefaultShellPath` - Default shell if not specified
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn Create(Identifier:u64, Name:String, OptionsValue:&Value, DefaultShellPath:String) -> Result<Self, String> {
		// Validate name length
		if Name.len() > MAX_TERMINAL_NAME_LENGTH {
			return Err(format!(
				"Terminal name exceeds maximum length of {} bytes",
				MAX_TERMINAL_NAME_LENGTH
			));
		}

		let ShellPath = OptionsValue
			.get("shellPath")
			.and_then(Value::as_str)
			.unwrap_or(&DefaultShellPath)
			.to_string();

		// Validate shell path length
		if ShellPath.len() > MAX_SHELL_PATH_LENGTH {
			return Err(format!("Shell path exceeds maximum length of {} bytes", MAX_SHELL_PATH_LENGTH));
		}

		let ShellArguments = match OptionsValue.get("shellArgs") {
			Some(Value::Array(Array)) => {
				let Args:Vec<String> = Array.iter().filter_map(Value::as_str).map(String::from).collect();

				// Validate argument count
				if Args.len() > MAX_SHELL_ARGUMENTS {
					return Err(format!("Shell arguments exceed maximum count of {}", MAX_SHELL_ARGUMENTS));
				}

				// Validate individual argument lengths
				for Arg in &Args {
					if Arg.len() > MAX_ARGUMENT_LENGTH {
						return Err(format!(
							"Shell argument exceeds maximum length of {} bytes",
							MAX_ARGUMENT_LENGTH
						));
					}
				}

				Args
			},

			_ => Vec::new(),
		};

		let CWD = OptionsValue.get("cwd").and_then(Value::as_str).map(PathBuf::from);

		// A more complete implementation would parse the `env` object.
		let EnvVars = None;

		Ok(Self {
			Identifier,
			Name,
			ShellPath,
			ShellArguments,
			CurrentWorkingDirectory:CWD,
			EnvironmentVariables:EnvVars,
			OSProcessIdentifier:None,
			IsPTY:true,
			PTYInputTransmitter:None,
			ReaderTaskHandle:None,
			ProcessWaitHandle:None,
		})
	}

	/// Checks if the terminal process is currently running.
	pub fn IsRunning(&self) -> bool { self.OSProcessIdentifier.is_some() }

	/// Checks if the terminal has an active PTY input channel.
	pub fn HasInputChannel(&self) -> bool { self.PTYInputTransmitter.is_some() }

	/// Returns the working directory as a string, or default if not set.
	pub fn GetWorkingDirectory(&self) -> String {
		self.CurrentWorkingDirectory
			.as_ref()
			.and_then(|Path| Path.to_str())
			.unwrap_or("")
			.to_string()
	}

	/// Clears the runtime handles (useful when terminating terminal).
	pub fn ClearHandles(&mut self) {
		self.PTYInputTransmitter = None;
		self.ReaderTaskHandle = None;
		self.ProcessWaitHandle = None;
	}
}
