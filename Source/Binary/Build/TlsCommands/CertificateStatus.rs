#![allow(non_snake_case)]

//! TLS certificate status snapshot returned by
//! `tls_check_cert_status`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStatus {

	pub exists:bool,

	pub is_valid:bool,

	pub days_until_expiry:i64,

	pub needs_renewal:bool,

	pub valid_until:String,
}
