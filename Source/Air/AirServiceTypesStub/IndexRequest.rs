//! `IndexFiles` request DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub request_id:String,

	pub paths:Vec<String>,

	pub recursive:bool,
}
