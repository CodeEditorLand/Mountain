//! # TerminalStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single active
//! integrated terminal instance.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde_json::Value;
use tokio::{
	sync::{Mutex as TokioMutex, mpsc as TokioMPSC},
	task::JoinHandle,
};

/// Holds the complete state and runtime resources for a single pseudo-terminal
/// (PTY) instance. This includes configuration, process identifiers, and
/// handles for I/O tasks.
#[derive(Debug, Clone)]
pub struct TerminalStateDTO {
	// --- Identifiers ---
	pub Identifier:u64,

	pub Name:String,

	pub OSProcessIdentifier:Option<u32>,

	// --- Configuration ---
	pub ShellPath:String,

	pub ShellArguments:Vec<String>,

	pub CurrentWorkingDirectory:Option<PathBuf>,

	pub EnvironmentVariables:Option<HashMap<String, Option<String>>>,

	pub IsPTY:bool,

	// --- Runtime Handles ---
	pub PTYInputTransmitter:Option<TokioMPSC::Sender<String>>,

	pub ReaderTaskHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,

	pub ProcessWaitHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
}

impl TerminalStateDTO {
	/// Creates a new `TerminalStateDTO` by parsing terminal options from a
	/// `serde_json::Value`.
	pub fn Create(Identifier:u64, Name:String, OptionsValue:&Value, DefaultShellPath:String) -> Self {
		let ShellPath = OptionsValue
			.get("shellPath")
			.and_then(Value::as_str)
			.unwrap_or(&DefaultShellPath)
			.to_string();

		let ShellArguments = match OptionsValue.get("shellArgs") {
			Some(Value::Array(Array)) => Array.iter().filter_map(Value::as_str).map(String::from).collect(),

			_ => Vec::new(),
		};

		let CWD = OptionsValue.get("cwd").and_then(Value::as_str).map(PathBuf::from);

		// A more complete implementation would parse the `env` object.
		let EnvVars = None;

		Self {
			Identifier,

			Name,

			ShellPath,

			ShellArguments,

			CurrentWorkingDirectory:CWD,

			EnvironmentVariables:EnvVars,

			OSProcessIdentifier:None,

			// Assume all terminals are PTYs for now
			IsPTY:true,

			PTYInputTransmitter:None,

			ReaderTaskHandle:None,

			ProcessWaitHandle:None,
		}
	}
}
