pub mod New;
pub mod NewUnix;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Protocol:String,

	pub Address:String,

	pub Port:u16,

	pub Path:Option<String>,
}
