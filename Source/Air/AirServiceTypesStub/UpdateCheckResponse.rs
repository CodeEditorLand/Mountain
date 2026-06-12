//! `CheckForUpdates` response DTO.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub update_available:bool,

	pub version:String,

	pub download_url:String,

	pub release_notes:String,

	pub error:String,
}
