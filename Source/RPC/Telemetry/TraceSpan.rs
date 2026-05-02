#![allow(non_snake_case)]

//! OTEL trace-span DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub trace_id:String,
	pub span_id:String,
	pub parent_span_id:Option<String>,
	pub name:String,
	pub start_time:i64,
	pub end_time:Option<i64>,
}
