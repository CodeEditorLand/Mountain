//! `IndexFiles` response DTO.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub success:bool,

	pub files_indexed:u32,

	pub total_size:u64,

	pub error:String,
}
