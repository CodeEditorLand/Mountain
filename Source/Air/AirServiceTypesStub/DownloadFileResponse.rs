//! `DownloadFile` response DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub success:bool,

	pub file_path:String,

	pub file_size:u64,

	pub checksum:String,

	pub error:String,
}
