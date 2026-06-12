//! `AuthenticateUser` response DTO.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub success:bool,

	pub token:String,

	pub error:String,
}
