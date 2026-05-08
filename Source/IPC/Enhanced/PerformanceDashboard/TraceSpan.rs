#![allow(non_snake_case)]

//! Distributed trace span: trace + span ids, parent linkage,
//! operation name, start / end / duration, tag bag, embedded
//! `TraceLog::Struct` entries.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::PerformanceDashboard::TraceLog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub trace_id:String,

	pub span_id:String,

	pub parent_span_id:Option<String>,

	pub operation_name:String,

	pub start_time:u64,

	pub end_time:Option<u64>,

	pub duration_ms:Option<u64>,

	pub tags:HashMap<String, String>,

	pub logs:Vec<TraceLog::Struct>,
}
