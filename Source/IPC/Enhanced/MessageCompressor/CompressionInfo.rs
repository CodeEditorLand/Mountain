#![allow(non_snake_case)]

//! Per-batch compression metadata - algorithm name, level,
//! and the achieved ratio. `none()` produces the
//! uncompressed sentinel (`algorithm:"none"`, `ratio:1.0`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub algorithm:String,
	pub level:u32,
	pub ratio:f64,
}

impl Struct {
	pub(super) fn none() -> Self { Self { algorithm:"none".to_string(), level:0, ratio:1.0 } }
}
