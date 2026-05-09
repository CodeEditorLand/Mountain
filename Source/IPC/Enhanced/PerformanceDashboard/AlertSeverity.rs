#![allow(non_snake_case)]

//! Severity tag for a `PerformanceAlert::Struct`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	Low,

	Medium,

	High,

	Critical,
}
