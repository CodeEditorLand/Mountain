//! Configuration read/write service.
use std::collections::HashMap;

pub struct Struct {
	config:HashMap<String, serde_json::Value>,
}

impl Struct {
	pub fn new() -> Self { Self { config:HashMap::new() } }

	pub fn get(&self, Key:&str) -> Option<&serde_json::Value> { self.config.get(Key) }

	pub fn set(&mut self, Key:String, Value:serde_json::Value) { self.config.insert(Key, Value); }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
