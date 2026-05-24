pub mod New;
pub mod HasRole;
pub mod HasPermission;
pub mod IpcDefault;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub user_id:String,

	pub roles:Vec<String>,

	pub permissions:Vec<String>,

	pub ip_address:String,

	pub timestamp:std::time::SystemTime,
}
