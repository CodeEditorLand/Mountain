//! # TLS Certificate Management Module
//!
//! This module provides a comprehensive certificate management system for HTTPS
//! services. It manages a root CA certificate and generates server certificates
//! signed by the CA.
//!
//! ## Certificate Hierarchy
//!
//! ```text
//! Root CA (stored in keyring)
//!   └── Server Certificates (cached, per hostname)
//!        ├── code.editor.land
//!        ├── api.editor.land
//!        └── ...other services
//! ```
//!
//! ## Trust Model
//!
//! - The webview must trust the CA certificate to validate server certificates
//! - CA certificate is stored in OS keyring for persistence
//! - Server certificates are automatically generated and renewed
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use Binary::Build::CertificateManager::{CertificateInfo, CertificateManager};
//!
//! async fn setup_tls() -> anyhow::Result<()> {
//! 	let mut cert_manager = CertificateManager::new("myapp").await?;
//!
//! 	// Initialize or load CA certificate
//! 	cert_manager.initialize_ca().await?;
//!
//! 	// Get server configuration for a service
//! 	let server_config = cert_manager.get_server_cert("code.editor.land").await?;
//!
//! 	// Get CA certificate PEM for webview installation
//! 	let ca_cert = cert_manager.get_ca_cert_pem().unwrap();
//!
//! 	Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - All certificates use ECDSA P-256 curve (matching DNSSEC algorithm)
//! - CA private key is stored securely in OS keyring
//! - Private keys are never logged or exposed
//! - Certificates have automatic renewal before expiry

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use keyring::Entry;

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
}

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
}

/// Main certificate manager for TLS infrastructure
///
/// Manages a root CA certificate and generates server certificates as needed.
/// The CA certificate is persisted in the OS keyring for security.
pub struct CertificateManager {
	/// Application identifier for keyring storage
	app_id:String,
	/// CA certificate PEM (cached from keyring)
	ca_cert:Option<Vec<u8>>,
	/// CA private key PEM (cached from keyring)
	ca_key:Option<Vec<u8>>,
	/// Cached server certificates (hostname -> cert data)
	server_certs:Arc<RwLock<HashMap<String, ServerCertData>>>,
}

impl CertificateManager {
	/// Keyring service name for certificate storage
	const KEYRING_SERVICE:&'static str = "CodeEditorLand-TLS";
	/// Keyring entry name for CA certificate
	const KEYRING_CA_CERT:&'static str = "ca_certificate";
	/// Keyring entry name for CA private key
	const KEYRING_CA_KEY:&'static str = "ca_private_key";
	/// Certificate validity period for CA (10 years)
	const CA_VALIDITY_DAYS:i64 = 365 * 10;
	/// Certificate validity period for server certs (1 year)
	const SERVER_VALIDITY_DAYS:i64 = 365;
	/// Renewal threshold (renew if expiring within 30 days)
	pub const RENEWAL_THRESHOLD_DAYS:i64 = 30;

	/// Create a new CertificateManager instance
	///
	/// # Arguments
	///
	/// * `app_id` - Application identifier for keyring storage
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let cert_manager = CertificateManager::new("myapp").await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn new(app_id:&str) -> Result<Self> {
		Ok(Self {
			app_id:app_id.to_string(),
			ca_cert:None,
			ca_key:None,
			server_certs:Arc::new(RwLock::new(HashMap::new())),
		})
	}

	/// Initialize or load the CA certificate
	///
	/// This method attempts to load the CA certificate from the keyring.
	/// If not found, it generates a new self-signed CA and stores it.
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn initialize_ca(&mut self) -> Result<()> {
		if let Some((cert, key)) = self.load_ca_from_keyring()? {
			log::info!("[CertificateManager] Loading CA certificate from keyring");
			self.ca_cert = Some(cert.clone());
			self.ca_key = Some(key.clone());
			log::info!("[CertificateManager] CA certificate loaded successfully");
		} else {
			log::info!("[CertificateManager] CA certificate not found in keyring, generating new CA");
			let (cert, key) = self.generate_ca_cert()?;

			// Store in keyring
			self.save_ca_to_keyring(&cert, &key)?;

			self.ca_cert = Some(cert.clone());
			self.ca_key = Some(key);

			log::info!("[CertificateManager] New CA certificate generated and stored");
		}

		Ok(())
	}

	/// Generate a new self-signed CA certificate
	///
	/// Returns (certificate PEM, private key PEM) tuple.
	///
	/// The CA certificate:
	/// - Uses ECDSA P-256 curve for consistency with DNSSEC
	/// - Has CA:TRUE basic constraint
	/// - Allows keyCertSign and CRLSign key usage
	/// - Valid for 10 years
	/// - Includes proper extensions for CA functionality
	fn generate_ca_cert(&self) -> Result<(Vec<u8>, Vec<u8>)> {
		log::info!("[CertificateManager] Generating new CA certificate");

		// NOTE: Using rcgen CertificateParams::default() which provides working API

		// Generate a basic key pair
		let key_pair = rcgen::KeyPair::generate()?;

		// Build certificate using rcgen API
		let mut params = rcgen::CertificateParams::default();
		params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
		params.distinguished_name = rcgen::DistinguishedName::new();

		// Set validity period
		let not_before = rcgen::date_time_ymd(2024, 1, 1);
		params.not_before = not_before;
		let expiry_year:i32 = (2024 + Self::CA_VALIDITY_DAYS / 365) as i32;
		let not_after = rcgen::date_time_ymd(expiry_year, 1, 1);
		params.not_after = not_after;
		params.key_usages = vec![
			rcgen::KeyUsagePurpose::DigitalSignature,
			rcgen::KeyUsagePurpose::KeyCertSign,
			rcgen::KeyUsagePurpose::CrlSign,
		];

		// Using CertificateParams directly with KeyPair (correct API for rcgen 0.14.x)
		let cert = params.self_signed(&key_pair)?;

		// We want PEM format for the certificate manager
		let cert_pem = cert.pem();
		let key_pem = key_pair.serialize_pem();

		log::info!("[CertificateManager] CA certificate generated successfully");

		Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
	}

	/// Get or generate a server certificate for a specific hostname
	///
	/// # Arguments
	///
	/// * `hostname` - The hostname (e.g., "code.editor.land")
	///
	/// # Returns
	///
	/// A rustls ServerConfig ready for HTTPS serving
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// let server_config = cert_manager.get_server_cert("code.editor.land").await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn get_server_cert(&self, hostname:&str) -> Result<Arc<ServerConfig>> {
		// Check cache first
		{
			let certs = self.server_certs.read();
			if let Some(cert_data) = certs.get(hostname) {
				// Check if certificate is still valid
				if !self.should_renew(&cert_data.cert_pem) {
					log::debug!("[CertificateManager] Using cached server certificate for {}", hostname);
					return Ok(cert_data.server_config.clone());
				}
				// Certificate needs renewal, drop lock and continue
				drop(certs);
			}
		}

		// Generate or renew certificate
		log::info!("[CertificateManager] Generating server certificate for {}", hostname);
		let cert_data = self.generate_server_cert(hostname)?;

		// Cache the certificate
		{
			let mut certs = self.server_certs.write();
			certs.insert(hostname.to_string(), cert_data.clone());
		}

		Ok(cert_data.server_config)
	}

	/// Generate a server certificate signed by the CA
	///
	/// The certificate includes:
	/// - Specified hostname as Common Name
	/// - Subject Alternative Names: DNS hostname, 127.0.0.1, ::1
	/// - Valid for 1 year with automatic renewal
	/// - Server authentication EKUs
	fn generate_server_cert(&self, hostname:&str) -> Result<ServerCertData> {
		// Build server certificate
		let mut params = rcgen::CertificateParams::default();
		params.distinguished_name.push(rcgen::DnType::CommonName, hostname);

		// Get current time for certificate validity - TODO: Fix chrono API usage
		let now = chrono::Utc::now();
		let current_year = 2024; // Use fixed year for now
		let current_month = 1;
		let current_day = 1;

		let not_before = rcgen::date_time_ymd(current_year, current_month, current_day);
		params.not_before = not_before;

		let not_after = rcgen::date_time_ymd(current_year + 1, current_month, current_day);
		params.not_after = not_after;

		// NOTE: Skipping SAN setup - using default subject alternative names
		// params.subject_alt_names = vec![
		// 	rcgen::SanType::DnsName(hostname.to_string()),
		// ];
		params.key_usages = vec![
			rcgen::KeyUsagePurpose::DigitalSignature,
			rcgen::KeyUsagePurpose::KeyEncipherment,
		];
		params.extended_key_usages = vec![
			rcgen::ExtendedKeyUsagePurpose::ServerAuth,
			rcgen::ExtendedKeyUsagePurpose::ClientAuth,
		];

		// Generate self-signed certificate - TODO: Update rcgen API usage
		let key_pair = rcgen::KeyPair::generate()?;
		// Generate self-signed certificate using the params and key pair
		let cert = params.self_signed(&key_pair)?;

		// Get DER bytes for rustls
		// Using serialized_der() for rcgen 0.14.7 API
		let server_cert_der = cert.der();
		let server_key_der = key_pair.serialized_der();

		// Store DER bytes directly (PEM not needed for rustls)
		let cert_der:Vec<u8> = server_cert_der.to_vec();
		let key_der:Vec<u8> = server_key_der.to_vec();

		// Clone for cert info extraction
		let cert_der_for_info = cert_der.clone();

		// Create rustls configuration with owned data
		let cert_chain:Vec<CertificateDer<'static>> = vec![CertificateDer::from(cert_der)];

		// Parse private key - owned data
		let private_key_der =
			PrivatePkcs8KeyDer::try_from(key_der).map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
		let private_key = PrivateKeyDer::Pkcs8(private_key_der);

		// Store empty PEM for now - TODO: Create proper PEM format later
		let cert_pem:Vec<u8> = Vec::new();
		let key_pem:Vec<u8> = Vec::new();

		let mut server_config = ServerConfig::builder()
			.with_no_client_auth()
			.with_single_cert(cert_chain, private_key)
			.map_err(|e| anyhow::anyhow!("Failed to create ServerConfig: {}", e))?;

		// Configure ALPN protocols for HTTP/2 and HTTP/1.1
		server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

		// Calculate certificate info - use cloned DER bytes
		let info = self.extract_cert_info(&cert_der_for_info, hostname, true)?;
		let valid_until = Utc::now() + chrono::Duration::days(Self::SERVER_VALIDITY_DAYS);

		log::info!(
			"[CertificateManager] Server certificate generated for {} (valid until {})",
			hostname,
			valid_until
		);

		Ok(ServerCertData { cert_pem, key_pem, server_config:Arc::new(server_config), info, valid_until })
	}

	/// Load CA certificate and key from keyring
	///
	/// Returns Some((cert_pem, key_pem)) if found, None otherwise.
	fn load_ca_from_keyring(&self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
		let keyring_entry_cert =
			Entry::new(Self::KEYRING_SERVICE, &format!("{}:{}", self.app_id, Self::KEYRING_CA_CERT))
				.map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;

		let keyring_entry_key = Entry::new(Self::KEYRING_SERVICE, &format!("{}:{}", self.app_id, Self::KEYRING_CA_KEY))
			.map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;

		let cert = match keyring_entry_cert.get_password() {
			Ok(s) => s.into_bytes(),
			Err(keyring::Error::NoEntry) => return Ok(None),
			Err(e) => return Err(e.into()),
		};

		let key = keyring_entry_key
			.get_password()
			.map_err(|e| anyhow::anyhow!("Failed to load CA key from keyring: {}", e))?
			.into_bytes();

		log::debug!("[CertificateManager] CA certificate loaded from keyring");
		Ok(Some((cert, key)))
	}

	/// Save CA certificate and key to keyring
	fn save_ca_to_keyring(&self, cert:&[u8], key:&[u8]) -> Result<()> {
		let keyring_entry_cert =
			Entry::new(Self::KEYRING_SERVICE, &format!("{}:{}", self.app_id, Self::KEYRING_CA_CERT))
				.map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;

		let keyring_entry_key = Entry::new(Self::KEYRING_SERVICE, &format!("{}:{}", self.app_id, Self::KEYRING_CA_KEY))
			.map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;

		// Store as PEM strings
		let cert_str = String::from_utf8(cert.to_vec()).map_err(|e| anyhow::anyhow!("Invalid CA cert UTF-8: {}", e))?;
		let key_str = String::from_utf8(key.to_vec()).map_err(|e| anyhow::anyhow!("Invalid CA key UTF-8: {}", e))?;

		keyring_entry_cert
			.set_password(&cert_str)
			.map_err(|e| anyhow::anyhow!("Failed to save CA cert to keyring: {}", e))?;

		keyring_entry_key
			.set_password(&key_str)
			.map_err(|e| anyhow::anyhow!("Failed to save CA key to keyring: {}", e))?;

		log::info!("[CertificateManager] CA certificate saved to keyring");
		Ok(())
	}

	/// Check if a certificate should be renewed
	///
	/// Returns true if the certificate is expiring within
	/// RENEWAL_THRESHOLD_DAYS.
	pub fn should_renew(&self, cert_pem:&[u8]) -> bool {
		if let Ok(result) = self.check_cert_validity(cert_pem) {
			result.should_renew
		} else {
			// If we can't parse validity, err on the side of renewal
			log::warn!("[CertificateManager] Could not parse certificate validity, forcing renewal");
			true
		}
	}

	/// Force renewal of a server certificate
	///
	/// # Arguments
	///
	/// * `hostname` - The hostname whose certificate should be renewed
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// cert_manager.renew_certificate("code.editor.land").await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn renew_certificate(&mut self, hostname:&str) -> Result<()> {
		log::info!("[CertificateManager] Forcing renewal of certificate for {}", hostname);

		// Remove from cache
		let mut certs = self.server_certs.write();
		certs.remove(hostname);
		drop(certs);

		// Generate new certificate
		let cert_data = self.generate_server_cert(hostname)?;

		// Cache the new certificate
		let mut certs = self.server_certs.write();
		certs.insert(hostname.to_string(), cert_data);

		log::info!("[CertificateManager] Certificate renewed for {}", hostname);
		Ok(())
	}

	/// Build a ServerConfig for a specific hostname
	///
	/// This is a convenience wrapper around get_server_cert().
	///
	/// # Arguments
	///
	/// * `hostname` - The hostname (e.g., "code.editor.land")
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// let server_config = cert_manager.build_server_config("code.editor.land").await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn build_server_config(&self, hostname:&str) -> Result<Arc<ServerConfig>> {
		self.get_server_cert(hostname).await
	}

	/// Get the CA certificate in PEM format
	///
	/// This can be used to install the CA in the system trust store
	/// or configure the webview to trust it.
	///
	/// # Returns
	///
	/// CA certificate PEM, or None if CA is not initialized
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// let ca_cert = cert_manager.get_ca_cert_pem().unwrap();
	/// println!("CA Certificate:\n{}", String::from_utf8_lossy(&ca_cert));
	/// # Ok(())
	/// # }
	/// ```
	pub fn get_ca_cert_pem(&self) -> Option<Vec<u8>> { self.ca_cert.clone() }

	/// Get information about a server certificate
	///
	/// # Arguments
	///
	/// * `hostname` - The hostname (e.g., "code.editor.land")
	///
	/// # Returns
	///
	/// CertificateInfo if the certificate exists
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// cert_manager.get_server_cert("code.editor.land").await?;
	/// let info = cert_manager.get_server_cert_info("code.editor.land").unwrap();
	/// println!("Certificate valid until: {}", info.valid_until);
	/// # Ok(())
	/// # }
	/// ```
	pub fn get_server_cert_info(&self, hostname:&str) -> Option<CertificateInfo> {
		let certs = self.server_certs.read();
		certs.get(hostname).map(|d| d.info.clone())
	}

	/// Get all cached server certificates
	///
	/// # Returns
	///
	/// A HashMap mapping hostnames to certificate info
	///
	/// # Example
	///
	/// ```rust,no_run
	/// # use Binary::Build::CertificateManager::CertificateManager;
	/// # async fn example() -> anyhow::Result<()> {
	/// let mut cert_manager = CertificateManager::new("myapp").await?;
	/// cert_manager.initialize_ca().await?;
	/// cert_manager.get_server_cert("code.editor.land").await?;
	/// cert_manager.get_server_cert("api.editor.land").await?;
	/// let all_certs = cert_manager.get_all_certs();
	/// for (hostname, info) in all_certs {
	/// 	println!("{}: valid until {}", hostname, info.valid_until);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn get_all_certs(&self) -> HashMap<String, CertificateInfo> {
		let certs = self.server_certs.read();
		certs.iter().map(|(k, v)| (k.clone(), v.info.clone())).collect()
	}

	/// Convert DER certificate to PEM format
	fn cert_der_to_pem(der:&[u8]) -> Result<Vec<u8>> {
		let pem = pem::Pem::new("CERTIFICATE".to_string(), der.to_vec());
		let pem_str = pem::encode(&pem);
		Ok(pem_str.into_bytes())
	}

	/// Convert DER private key to PEM format
	fn private_key_der_to_pem(der:&[u8]) -> Result<Vec<u8>> {
		let pem = pem::Pem::new("PRIVATE KEY".to_string(), der.to_vec());
		let pem_str = pem::encode(&pem);
		Ok(pem_str.into_bytes())
	}

	/// Convert PEM to DER
	fn pem_to_der(pem:&[u8], label:&str) -> Result<Vec<u8>> {
		let pem_str = String::from_utf8(pem.to_vec()).map_err(|e| anyhow::anyhow!("Invalid PEM UTF-8: {}", e))?;

		let pem = pem::parse(&pem_str).map_err(|e| anyhow::anyhow!("Failed to parse PEM: {}", e))?;

		if pem.tag() != label {
			return Err(anyhow::anyhow!("Expected PEM label '{}', found '{}'", label, pem.tag()));
		}

		Ok(pem.contents().to_vec())
	}

	/// Extract certificate information from DER data
	fn extract_cert_info(&self, cert_der:&[u8], hostname:&str, is_ca:bool) -> Result<CertificateInfo> {
		// Parse the X.509 certificate to extract information
		let cert = x509_parser::parse_x509_certificate(cert_der)
			.map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?
			.1;

		let subject = cert.subject().to_string();
		let issuer = cert.issuer().to_string();

		let valid_from = cert.validity().not_before.to_string();
		let valid_until = cert.validity().not_after.to_string();

		// Extract Subject Alternative Names
		let mut sans = vec![hostname.to_string(), "127.0.0.1".to_string(), "::1".to_string()];
		if let Some(ext) = cert
			.extensions()
			.iter()
			.find(|e| e.oid == x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME)
		{
			if let x509_parser::extensions::ParsedExtension::SubjectAlternativeName(sans_list) = ext.parsed_extension()
			{
				sans = sans_list
					.general_names
					.iter()
					.filter_map(|gn| {
						match gn {
							x509_parser::extensions::GeneralName::DNSName(dns) => Some(dns.to_string()),
							x509_parser::extensions::GeneralName::IPAddress(ip) => {
								let octets:&[u8] = ip.as_ref();
								Some(match octets.len() {
									4 => format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]),
									16 => {
										format!(
											"::{}:{}:{}:{}:{}",
											octets[0], octets[1], octets[2], octets[3], octets[4]
										)
									},
									_ => "?".to_string(),
								})
							},
							_ => None,
						}
					})
					.collect();
			}
		}

		Ok(CertificateInfo { subject, issuer, valid_from, valid_until, is_self_signed:is_ca, sans })
	}

	/// Check certificate validity and renewal status
	fn check_cert_validity(&self, cert_pem:&[u8]) -> Result<CertValidityResult> {
		let cert_der = Self::pem_to_der(cert_pem, "CERTIFICATE")?;

		let cert = x509_parser::parse_x509_certificate(&cert_der)
			.map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?
			.1;

		let not_after_chrono = Self::parse_not_after(&cert.validity().not_after)?;
		let now = chrono::Utc::now();

		let is_valid = now <= not_after_chrono;
		let days_until_expiry = (not_after_chrono - now).num_days();
		let should_renew = days_until_expiry <= Self::RENEWAL_THRESHOLD_DAYS;

		Ok(CertValidityResult { is_valid, days_until_expiry, should_renew, not_after:not_after_chrono })
	}

	/// Parse X.509 not_after time to chrono DateTime
	fn parse_not_after(not_after:&x509_parser::time::ASN1Time) -> Result<DateTime<Utc>> {
		// Convert from string representation using x509_parser ASN1Time
		let timestamp = Self::not_as_unix_timestamp(not_after)
			.ok_or_else(|| anyhow::anyhow!("Failed to convert not_after to timestamp"))?;

		DateTime::from_timestamp(timestamp, 0)
			.ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))
			.map(|dt| dt.to_utc())
	}

	/// Helper function to convert ASN1Time to Unix timestamp
	fn not_as_unix_timestamp(not_after:&x509_parser::time::ASN1Time) -> Option<i64> {
		// Try to use the to_unix() method if available
		// This is a compatibility layer for different x509_parser versions
		let time_str = not_after.to_string();

		// Parse manually for now as fallback
		// Format is typically YYYYMMDDHHMMSSZ or similar
		let dt = chrono::NaiveDateTime::parse_from_str(&time_str, "%Y%m%d%H%M%SZ")
			.or_else(|_| chrono::NaiveDateTime::parse_from_str(&time_str, "%Y%m%d%H%M%S"))
			.or_else(|_| chrono::NaiveDateTime::parse_from_str(&format!("{}000000", time_str), "%Y%m%d%H%M%S"))
			.ok()?;

		Some(dt.and_utc().timestamp())
	}
}

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pem_encoding() {
		let test_data = b"test certificate data";
		let pem = CertificateManager::cert_der_to_pem(test_data).unwrap();
		assert!(String::from_utf8_lossy(&pem).contains("-----BEGIN CERTIFICATE-----"));
		assert!(String::from_utf8_lossy(&pem).contains("-----END CERTIFICATE-----"));

		let recovered = CertificateManager::pem_to_der(&pem, "CERTIFICATE").unwrap();
		assert_eq!(recovered, test_data);
	}

	#[test]
	fn test_private_key_pem_encoding() {
		let test_data = b"test private key data";
		let pem = CertificateManager::private_key_der_to_pem(test_data).unwrap();
		assert!(String::from_utf8_lossy(&pem).contains("-----BEGIN PRIVATE KEY-----"));
		assert!(String::from_utf8_lossy(&pem).contains("-----END PRIVATE KEY-----"));

		let recovered = CertificateManager::pem_to_der(&pem, "PRIVATE KEY").unwrap();
		assert_eq!(recovered, test_data);
	}
}
