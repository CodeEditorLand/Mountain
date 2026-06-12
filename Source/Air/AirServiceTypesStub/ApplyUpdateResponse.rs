//! `ApplyUpdate` response DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub success:bool,

	pub error:String,
}
