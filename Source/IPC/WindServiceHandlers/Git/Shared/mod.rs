//! Shared state and utilities for `Git/*` atomic handlers.

use std::{
	collections::HashMap,
	sync::{Mutex, OnceLock},
};

pub(crate) fn running_processes() -> &'static Mutex<HashMap<String, u32>> {
	static SLOT:OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();

	SLOT.get_or_init(|| Mutex::new(HashMap::new()))
}

pub mod AsStringArray;

pub mod ClearPid;

pub mod RegisterPid;

pub mod ResolveCwd;

pub mod RunGit;

pub mod RunningProcesses;

pub mod TakePid;
