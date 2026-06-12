//! Command registration and execution service.
use std::collections::HashMap;

/// Command registration and execution service.
pub struct Struct {
	commands:HashMap<String, String>,
}

/// Creates a new command service.
impl Struct {
	pub fn new() -> Self { Self { commands:HashMap::new() } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
