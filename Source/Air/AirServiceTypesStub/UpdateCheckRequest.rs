//! `CheckForUpdates` request DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,

	pub current_version:String,

	pub channel:String,
}
