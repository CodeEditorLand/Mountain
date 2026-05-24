//! `TerminalStateDTO::GetWorkingDirectory`

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

pub fn Fn(This:&Struct) -> String {
		This.CurrentWorkingDirectory
			.as_ref()
			.and_then(|Path| Path.to_str())
			.unwrap_or("")
			.to_string()
	}
