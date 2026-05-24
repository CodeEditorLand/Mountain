pub mod New;
pub mod Get;
pub mod Set;

use std::collections::HashMap;

pub struct Struct {
	config:HashMap<String, serde_json::Value>,
}
