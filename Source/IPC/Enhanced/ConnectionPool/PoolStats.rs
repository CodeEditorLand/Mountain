#![allow(non_snake_case)]

//! Aggregated pool counters surfaced to the dashboard - total
//! / active / idle / healthy connection counts, queue size,
//! average wait time, total / successful operation tallies,
//! and the rolled-up error rate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub total_connections:usize,

	pub active_connections:usize,

	pub idle_connections:usize,

	pub healthy_connections:usize,

	pub max_connections:usize,

	pub min_connections:usize,

	pub wait_queue_size:usize,

	pub average_wait_time_ms:f64,

	pub total_operations:u64,

	pub successful_operations:u64,

	pub error_rate:f64,
}
