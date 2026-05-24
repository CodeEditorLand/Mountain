pub mod New;
pub mod InitializeCa;
pub mod GetServerCert;
pub mod ShouldRenew;
pub mod RenewCertificate;
pub mod BuildServerConfig;
pub mod GetCaCertPem;
pub mod GetServerCertInfo;
pub mod GetAllCerts;

use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring_core::{Entry, Error as KeyringError};
use crate::dev_log;

/// Certificate information for display and validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertificateInfo {
	/// Subject Common Name (e.g., "CN=localhost")
	pub subject:String,

	/// Issuer Common Name (for self-signed, same as subject)
	pub issuer:String,

	/// Validity start time (ISO 8601)
	pub valid_from:String,

	/// Validity end time (ISO 8601)
	pub valid_until:String,

	/// Whether this is a self-signed certificate
	pub is_self_signed:bool,

	/// Subject Alternative Names
	pub sans:Vec<String>,

/// Server certificate data including PEM formats and rustls configuration
#[derive(Clone)]
struct ServerCertData {
	/// Certificate in PEM format
	cert_pem:Vec<u8>,

	/// Private key in PEM format
	key_pem:Vec<u8>,

	/// rustls ServerConfig for serving TLS
	server_config:Arc<ServerConfig>,

	/// Certificate info
	info:CertificateInfo,

	/// Validity end time
	valid_until:DateTime<Utc>,

/// Main certificate manager for TLS infrastructure
/// Manages a root CA certificate and generates server certificates as needed.
/// The CA certificate is persisted in the OS keyring for security.
pub struct Struct {
	/// Application identifier for keyring storage
	app_id:String,

	/// CA certificate PEM (cached from keyring)
	ca_cert:Option<Vec<u8>>,

	/// CA private key PEM (cached from keyring)
	ca_key:Option<Vec<u8>>,

	/// Cached server certificates (hostname -> cert data)
	server_certs:Arc<RwLock<HashMap<String, ServerCertData>>>,

/// Certificate validity check result
#[derive(Debug, Clone)]
struct CertValidityResult {
	/// Whether the certificate is currently valid
	is_valid:bool,

	/// Days until expiry (negative if expired)
	days_until_expiry:i64,

	/// Whether renewal is recommended
	should_renew:bool,

	/// Certificate expiry time
	not_after:DateTime<Utc>,
}
}
}
}
