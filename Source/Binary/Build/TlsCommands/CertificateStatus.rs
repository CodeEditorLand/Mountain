//! TLS certificate status snapshot returned by
//! `TlsCheckCertStatus`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub exists:bool,

	pub is_valid:bool,

	pub days_until_expiry:i64,

	pub needs_renewal:bool,

	pub valid_until:String,
}
