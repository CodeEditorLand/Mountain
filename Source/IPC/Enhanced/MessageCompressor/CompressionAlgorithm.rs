
//! Wire-format selector for the compressor - Brotli, Gzip, or
//! Zlib. The compressor delegates to the matching encoder /
//! decoder pair.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Enum {
	Brotli,

	Gzip,

	Zlib,
}
