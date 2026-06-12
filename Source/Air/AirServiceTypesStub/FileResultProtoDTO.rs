//! Single file result inside `SearchFilesResponse`.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub path:String,

	pub size:u64,

	pub line:Option<u32>,

	pub content:Option<String>,
}
