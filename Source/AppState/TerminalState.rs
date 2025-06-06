
// Defines the data structure for representing the state of a single active
// terminal instance.

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

/// Represents the in-memory state of an active terminal.
#[derive(Debug, Clone)]
pub struct TerminalState {
	pub Identifier:u64,
	pub Name:String,
	pub ShellPath:String,
	pub ShellArgument:Vec<String>,
	pub CurrentWorkingDirectory:Option<PathBuf>,
	pub EnvironmentVariables:Option<HashMap<String, Option<String>>>, // Value can be None to unset
	pub OsProcessIdentifier:Option<u32>,
	pub IsPty:bool,
	// A channel for sending input from the application to the terminal's underlying process.
	#[serde(skip)]
	pub PtyInputTransmitter:Option<TokioMpsc::Sender<String>>,
	// A handle to the asynchronous task that reads output from the terminal process.
	#[serde(skip)]
	pub ReaderTaskHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
	// A handle to the asynchronous task that waits for the terminal process to exit.
	#[serde(skip)]
	pub ProcessWaitHandle:Option<Arc<TokioMutex<Option<JoinHandle<()>>>>>,
}

impl TerminalState {
	/// Creates a new `TerminalState` instance from creation options.
	pub fn New(Identifier:u64, Name:String, OptionsValue:&Value, DefaultShellPath:String) -> Self {
		let ShellPathOptionString = OptionsValue
			.get("shellPath")
			.or_else(|| OptionsValue.get("executable"))
			.and_then(Value::as_str);
		let FinalShellPath = ShellPathOptionString.map_or(DefaultShellPath, String::from);

		let ShellArgumentValue = OptionsValue.get("shellArgument").or_else(|| OptionsValue.get("args"));
		let FinalShellArgumentVector = if let Some(ArgumentString) = ShellArgumentValue.and_then(Value::as_str) {
			ArgumentString.split_whitespace().map(String::from).collect()
		} else if let Some(ArrayValue) = ShellArgumentValue.and_then(Value::as_array) {
			ArrayValue.iter().filter_map(Value::as_str).map(String::from).collect()
		} else {
			Vec::new()
		};

		let CurrentWorkingDirectoryOptionPath = OptionsValue.get("cwd").and_then(|CwdValue| {
			CwdValue
				.as_str()
				.map(PathBuf::from)
				.or_else(|| CwdValue.get("fsPath").and_then(Value::as_str).map(PathBuf::from))
		});

		let EnvironmentVariablesOptionMap =
			if let Some(EnvironmentMapValue) = OptionsValue.get("env").and_then(Value::as_object) {
				let mut EnvironmentMap = HashMap::new();
				for (Key, ValueValue) in EnvironmentMapValue {
					if let Some(ValueString) = ValueValue.as_str() {
						EnvironmentMap.insert(Key.clone(), Some(ValueString.to_string()));
					} else if ValueValue.is_null() {
						EnvironmentMap.insert(Key.clone(), None); // To unset the variable
					}
				}
				if EnvironmentMap.is_empty() { None } else { Some(EnvironmentMap) }
			} else {
				None
			};

		Self {
			Identifier,
			Name,
			ShellPath:FinalShellPath,
			ShellArgument:FinalShellArgumentVector,
			CurrentWorkingDirectory:CurrentWorkingDirectoryOptionPath,
			EnvironmentVariables:EnvironmentVariablesOptionMap,
			OsProcessIdentifier:None,
			IsPty:OptionsValue.get("isPty").and_then(Value::as_bool).unwrap_or(true),
			PtyInputTransmitter:None,
			ReaderTaskHandle:None,
			ProcessWaitHandle:None,
		}
	}
}
