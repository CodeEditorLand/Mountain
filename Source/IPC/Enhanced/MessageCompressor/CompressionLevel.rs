//! Compression strength dial - `Fast` (1), `Balanced` (6),
//! `High` (11). Maps directly to Brotli quality / flate2
//! compression level.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Enum {
	Fast = 1,

	Balanced = 6,

	High = 11,
}
