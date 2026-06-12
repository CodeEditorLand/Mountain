//! Command registration and execution service.
use std::collections::HashMap;

pub struct Struct {
	commands:HashMap<String, String>,
}

impl Struct {
	pub fn new() -> Self { Self { commands:HashMap::new() } }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
