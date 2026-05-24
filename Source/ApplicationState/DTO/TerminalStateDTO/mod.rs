pub mod Create;
pub mod IsRunning;
pub mod HasInputChannel;
pub mod GetWorkingDirectory;
pub mod ClearHandles;

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

/// Thread-safe handle around a portable-pty master PTY. We keep the handle
/// alive past CreateTerminal so Resize / drop-to-kill semantics work. Not
/// Clone / Serialize; the surrounding struct marks it `#[serde(skip)]`.
pub type PtyMasterHandle = Arc<StandardMutex<Box<dyn MasterPty + Send>>>;

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
/// `Debug` is implemented manually at the bottom of this file because the
/// `PTYMaster` field stores `dyn MasterPty + Send`, which does not itself
/// implement `Debug`. The manual impl prints the master handle as an opaque
/// placeholder so the surrounding struct remains `Debug`-printable.
#[derive(Clone, Serialize, Deserialize)]
pub struct Struct {
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

	/// Master PTY handle kept alive for `Resize` and for ownership semantics
	/// (dropping the master closes the slave, terminating the shell).
	#[serde(skip)]
	pub PTYMaster:Option<PtyMasterHandle>,
}
