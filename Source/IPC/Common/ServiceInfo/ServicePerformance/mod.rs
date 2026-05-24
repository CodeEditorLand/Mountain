pub mod New;
pub mod RecordRequest;
pub mod RecordError;
pub mod ErrorRate;

use std::time::Instant;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	pub RequestCount:u64,

	pub ErrorCount:u64,

	pub AverageResponseTimeMs:f64,

	#[serde(skip)]
	pub LastUpdated:Instant,
}
