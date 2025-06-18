// @module TerminalStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single active integrated terminal.

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex as StdMutex},
};

use serde_json::Value;
use tokio::{
	sync::{Mutex as TokioMutex, mpsc as TokioMpsc},
	task::JoinHandle,
};

// Holds the complete state and resources for a single pseudo-terminal (PTY)
// instance.
#[derive(Debug, Clone)]
pub struct TerminalStateDTO {
	// --- Identifiers ---
	pub Identifier:u64,
	pub Name:String,
	pub OsProcessIdentifier:Option<u32>,

	// --- Configuration ---
	pub ShellPath:String,
	pub ShellArgument:Vec<String>,
	pub CurrentWorkingDirectory:Option<PathBuf>,
	pub EnvironmentVariables:Option<HashMap<String, Option<String>>>,
	pub IsPty:bool,

	// --- Runtime Handles ---
	#[allow(clippy::type_complexity)]
	pub PtyInputTransmitter:Option<TokioMpsc::Sender<String>>,
	#[allow(clippy::type_complexity)]
	pub ReaderTaskHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
	#[allow(clippy::type_complexity)]
	pub ProcessWaitHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
}

impl TerminalStateDTO {
	// Creates a new `TerminalStateDTO` by parsing terminal options.
	pub fn New(identifier:u64, name:String, options_value:&Value, default_shell_path:String) -> Self {
		let shell_path = options_value
			.get("shellPath")
			.and_then(Value::as_str)
			.unwrap_or(&default_shell_path)
			.to_string();

		let shell_args = match options_value.get("shellArgs") {
			Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).map(String::from).collect(),
			_ => Vec::new(),
		};

		let cwd = options_value.get("cwd").and_then(Value::as_str).map(PathBuf::from);

		// A more complete implementation would parse the `env` object.
		let env_vars = None;

		Self {
			Identifier:identifier,
			Name:name,
			ShellPath:shell_path,
			ShellArgument:shell_args,
			CurrentWorkingDirectory:cwd,
			EnvironmentVariables:env_vars,
			OsProcessIdentifier:None,
			IsPty:true, // Assuming all terminals are PTYs for now
			PtyInputTransmitter:None,
			ReaderTaskHandle:None,
			ProcessWaitHandle:None,
		}
	}
}
