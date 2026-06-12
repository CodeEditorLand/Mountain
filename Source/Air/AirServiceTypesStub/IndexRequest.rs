//! `IndexFiles` request DTO.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub request_id:String,

	pub paths:Vec<String>,

	pub recursive:bool,
}
