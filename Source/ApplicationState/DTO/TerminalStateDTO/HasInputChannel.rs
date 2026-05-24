//! `TerminalStateDTO::HasInputChannel`

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

pub fn Fn(This:&Struct) -> bool { This.PTYInputTransmitter.is_some() }
