pub mod Success;
pub mod Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub CorrelationId:String,

	pub Data:serde_json::Value,

	pub Success:bool,

	pub Error:Option<String>,

	pub Timestamp:u64,
}
