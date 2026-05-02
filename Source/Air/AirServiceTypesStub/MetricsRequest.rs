#![allow(non_snake_case)]

//! `GetMetrics` request DTO.

#[derive(Debug, Clone)]
pub struct Struct {
	pub request_id:String,
	pub metric_type:Option<String>,
}
