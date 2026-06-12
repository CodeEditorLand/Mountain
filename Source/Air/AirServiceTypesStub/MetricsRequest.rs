//! `GetMetrics` request DTO.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub request_id:String,

	pub metric_type:Option<String>,
}
