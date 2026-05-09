#![allow(non_snake_case)]

//! Connection-pool tunables: max / min connection counts plus
//! the four millisecond budgets (acquire timeout, max
//! lifetime, idle timeout, health-check interval).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub max_connections:usize,

	pub min_connections:usize,

	pub connection_timeout_ms:u64,

	pub max_lifetime_ms:u64,

	pub idle_timeout_ms:u64,

	pub health_check_interval_ms:u64,
}

impl Default for Struct {

	fn default() -> Self {

		Self {

			max_connections:10,

			min_connections:2,

			connection_timeout_ms:30000,

			max_lifetime_ms:300000,

			idle_timeout_ms:60000,

			health_check_interval_ms:30000,
		}
	}
}
