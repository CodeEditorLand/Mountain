//! `AuthenticateUser` response DTO.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub success:bool,

	pub token:String,

	pub error:String,
}
