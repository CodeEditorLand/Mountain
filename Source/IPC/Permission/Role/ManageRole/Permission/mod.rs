pub mod New;
pub mod NewSensitive;
pub mod SetSensitive;
pub mod GetAction;
pub mod GetCategory;
pub mod Validate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Name:String,

	pub Description:String,

	pub Category:String,

	pub IsSensitive:bool,
}
