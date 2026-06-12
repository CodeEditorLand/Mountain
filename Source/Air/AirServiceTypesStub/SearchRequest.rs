//! `SearchFiles` request DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub request_id:String,

	pub query:String,

	pub file_patterns:Vec<String>,

	pub max_results:u32,
}
