//! Result envelope returned by `tls_generate_cert`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateGenerationResult {

	pub hostname:String,

	pub success:bool,

	pub valid_until:String,

	pub message:String,
}
