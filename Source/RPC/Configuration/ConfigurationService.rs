//! Configuration read/write service.
use std::collections::HashMap;

/// Configuration read/write service.
pub struct Struct {
	config:HashMap<String, serde_json::Value>,
}

/// Creates a new configuration service.
impl Struct {
	/// Creates a new empty configuration store.
	pub fn new() -> Self { Self { config:HashMap::new() } }

	/// Look up a configuration value by key.
	pub fn get(&self, Key:&str) -> Option<&serde_json::Value> { self.config.get(Key) }

	/// Set a configuration value by key.
	pub fn set(&mut self, Key:String, Value:serde_json::Value) { self.config.insert(Key, Value); }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
