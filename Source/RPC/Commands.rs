//! # Commands RPC Service
//!
//! Command registration and execution service.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Command service
pub struct CommandService {
	#[allow(dead_code)]
	commands:HashMap<String, String>,
}

impl CommandService {
	pub fn new() -> Self { Self { commands:HashMap::new() } }
}

impl Default for CommandService {
	fn default() -> Self { Self::new() }
}

/// Command validation
pub struct CommandValidation;

impl CommandValidation {
	pub fn new() -> Self { Self {} }
}

impl Default for CommandValidation {
	fn default() -> Self { Self::new() }
}

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
	pub id:String,
	pub title:String,
	pub description:Option<String>,
}
