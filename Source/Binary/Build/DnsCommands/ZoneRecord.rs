//! Single DNS zone record (name / type / TTL / data) returned
//! inside `ZoneInfo::Struct`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRecord {
	pub name:String,

	pub record_type:String,

	pub ttl:u32,

	pub data:String,
}
