//! TLS certificate status snapshot returned by
//! `tls_check_cert_status`.

use serde::{Deserialize, Serialize};

/// Certificate status snapshot with existence, validity, expiry timeline,
/// renewal flag, and the RFC 3339 expiry timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStatus {
	pub exists:bool,

	pub is_valid:bool,

	pub days_until_expiry:i64,

	pub needs_renewal:bool,

	pub valid_until:String,
}
