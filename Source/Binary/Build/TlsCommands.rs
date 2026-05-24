//! # TLS certificate management commands
//!
//! Tauri commands that expose the local CA + server cert cache
//! (managed by `CertificateManager`) to the webview. Each
//! command lives in its own sibling file; the wire-bound names
//! match the file names.
//!
//! Currently registered nowhere - kept for the upcoming
//! TLS-aware webview surface. Adding the entries to
//! `Binary/Main/Entry.rs::invoke_handler!` is the activation
//! step.

pub mod CertificateGenerationResult;

pub mod CertificateStatus;

pub mod TlsCheckCertStatus;

pub mod TlsDeleteCert;

pub mod TlsGenerateCert;

pub mod TlsGetAllCerts;

pub mod TlsGetCaCert;

pub mod TlsGetServerCertInfo;

pub mod TlsInitialize;

pub mod TlsRenewCertificate;

#[cfg(test)]
mod tests {

	use super::CertificateStatus::Struct;

	#[test]
	fn CertificateStatusSerialization() {
		let status = CertificateStatus {
			exists:true,

			is_valid:true,

			days_until_expiry:30,

			needs_renewal:true,

			valid_until:"2025-01-01T00:00:00Z".to_string(),
		};

		let json = serde_json::to_string(&status).unwrap();

		let deserialized:CertificateStatus = serde_json::from_str(&json).unwrap();

		assert_eq!(deserialized.exists, status.exists);
	}
}
