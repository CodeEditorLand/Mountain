//! `ApplyUpdate` request DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub request_id:String,

	pub update_id:String,

	pub update_path:String,
}
