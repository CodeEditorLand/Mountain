//! `TerminalStateDTO::Create`

use super::Struct;
use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex as StandardMutex},
};
use portable_pty::MasterPty;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
	sync::{Mutex as TokioMutex, mpsc as TokioMPSC},
	task::JoinHandle,
};

pub fn Fn(Identifier:u64, Name:String, OptionsValue:&Value, DefaultShellPath:String) -> Result<Self, String> {
		// Validate name length
		if Name.len() > MAX_TERMINAL_NAME_LENGTH {
			return Err(format!(
				"Terminal name exceeds maximum length of {} bytes",
				MAX_TERMINAL_NAME_LENGTH
			));
		}

		let ShellPath = OptionsValue
			.Get("shellPath")
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
			PTYMaster:None,
		})
	}
