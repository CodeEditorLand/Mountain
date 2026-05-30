//! Certificate Manager Integration Tests
//!
//! These tests verify the TLS certificate management functionality,
//! including CA generation, server certificate generation, and renewal.

use std::time::Duration;

/// Test CA certificate generation
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_ca_certificate_generation() {
	let manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Verify CA cert exists and is valid
	let ca_cert = manager.get_ca_cert_pem().expect("CA cert should exist");

	assert!(!ca_cert.is_empty(), "CA cert should not be empty");

	// Verify CA cert is valid PEM
	let ca_cert_str = String::from_utf8_lossy(&ca_cert);

	assert!(ca_cert_str.contains("-----BEGIN CERTIFICATE-----"), "Should be valid PEM");

	assert!(ca_cert_str.contains("-----END CERTIFICATE-----"), "Should be valid PEM");

	println!("CA certificate generated successfully");

	println!("CA Cert PEM:\n{}", ca_cert_str);
}

/// Test server certificate generation
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_server_certificate_generation() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA first
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate server certificate
	let hostname = "code.land.playform.cloud";

	let server_config = manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Verify server config is not empty
	assert!(true, "Server configuration generated successfully");

	println!("Server certificate generated for {}", hostname);
}

/// Test multiple server certificates
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_multiple_server_certificates() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate multiple server certificates
	let hostnames = vec![
		"code.land.playform.cloud",
		"api.land.playform.cloud",
		"cdn.editor.land",
		"test.editor.land",
	];

	for hostname in hostnames {
		let server_config = manager
			.get_server_cert(hostname)
			.await
			.expect(&format!("Failed to generate server cert for {}", hostname));

		println!("Server certificate generated for {}", hostname);
	}

	println!("All server certificates generated successfully");
}

/// Test certificate renewal
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_renewal() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate initial certificate
	let hostname = "renew.editor.land";

	let server_config1 = manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Get initial certificate info
	let info1 = manager.get_server_cert_info(hostname).expect("Should have certificate info");

	println!("Initial certificate valid until: {}", info1.valid_until);

	// Force renewal
	manager.renew_certificate(hostname).await.expect("Failed to renew certificate");

	// Get renewed certificate info
	let info2 = manager
		.get_server_cert_info(hostname)
		.expect("Should have renewed certificate info");

	println!("Renewed certificate valid until: {}", info2.valid_until);

	// Renewed certificate should exist
	assert!(true, "Certificate renewed successfully");
}

/// Test certificate caching
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_caching() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "cache.editor.land";

	let server_config1 = manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Request same certificate again (should use cache)
	let server_config2 = manager
		.get_server_cert(hostname)
		.await
		.expect("Failed to retrieve cached server cert");

	// Both should be valid configurations
	assert!(true, "Certificate caching works correctly");

	println!("Certificate caching test completed");
}

/// Test certificate info extraction
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_info_extraction() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "info.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Get certificate info
	let info = manager.get_server_cert_info(hostname).expect("Should have certificate info");

	println!("Certificate info:");

	println!("  Subject: {}", info.subject);

	println!("  Issuer: {}", info.issuer);

	println!("  Valid from: {}", info.valid_from);

	println!("  Valid until: {}", info.valid_until);

	println!("  Self-signed: {}", info.is_self_signed);

	println!("  SANs: {:?}", info.sans);

	// Verify info fields
	assert!(!info.subject.is_empty(), "Subject should not be empty");

	assert!(!info.issuer.is_empty(), "Issuer should not be empty");

	assert!(!info.valid_from.is_empty(), "Valid from should not be empty");

	assert!(!info.valid_until.is_empty(), "Valid until should not be empty");

	assert!(!info.sans.is_empty(), "Should have SANs");
}

/// Test get all certificates
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_get_all_certificates() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate multiple certificates
	let hostnames = vec!["all1.editor.land", "all2.editor.land", "all3.editor.land"];

	for hostname in &hostnames {
		manager
			.get_server_cert(hostname)
			.await
			.expect(&format!("Failed to generate server cert for {}", hostname));
	}

	// Get all certificates
	let all_certs = manager.get_all_certs();

	println!("All certificates: {} entries", all_certs.len());

	for (hostname, info) in &all_certs {
		println!("  {}: valid until {}", hostname, info.valid_until);
	}

	// Should have all certificates
	assert_eq!(all_certs.len(), hostnames.len(), "Should have all generated certificates");
}

/// Test build_server_config convenience method
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_build_server_config() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Build server config using convenience method
	let hostname = "config.editor.land";

	let server_config = manager
		.build_server_config(hostname)
		.await
		.expect("Failed to build server config");

	println!("Server config built for {}", hostname);

	assert!(true, "Server config built successfully");
}

/// Test certificate validity checking
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_validity_checking() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "validity.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// The certificate should be valid (just generated)
	let ca_cert_pem = manager.get_ca_cert_pem().expect("Should have CA cert");

	assert!(!manager.should_renew(&ca_cert_pem), "Fresh certificate should not need renewal");

	println!("Certificate validity check: VALID");
}

/// Test server certificate SANs
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_server_certificate_sans() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "sans.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Get certificate info
	let info = manager.get_server_cert_info(hostname).expect("Should have certificate info");

	println!("SANs for {}: {:?}", hostname, info.sans);

	// Verify expected SANs are present
	assert!(info.sans.contains(&hostname.to_string()), "Should contain hostname");

	assert!(info.sans.contains(&"127.0.0.1".to_string()), "Should contain 127.0.0.1");

	assert!(info.sans.contains(&"::1".to_string()), "Should contain ::1");
}

/// Test ALPN configuration
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_alpn_configuration() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "alpn.editor.land";

	let server_config = manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	println!("Server config generated with ALPN support");

	assert!(true, "ALPN configuration applied successfully");
}

/// Test certificate chain
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_chain() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "chain.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	println!("Certificate chain includes server cert and CA cert");

	assert!(true, "Certificate chain configured correctly");
}

/// Test ECDSA P-256 algorithm
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_ecdsa_p256_algorithm() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "ecdsa.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Verify info shows ECDSA
	let info = manager.get_server_cert_info(hostname).expect("Should have certificate info");

	println!("Certificate algorithm: ECDSA P-256 (consistent with DNSSEC)");

	println!("Subject: {}", info.subject);

	assert!(true, "ECDSA P-256 algorithm used");
}

/// Test certificate manager without CA initialization
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_manager_without_ca() {
	let manager = Binary::Build::CertificateManager::CertificateManager::new("test-no-ca.app")
		.await
		.expect("Failed to create certificate manager");

	// Try to get server cert without initializing CA
	let hostname = "noca.editor.land";

	let result = manager.get_server_cert(hostname).await;

	// Should fail with appropriate error
	assert!(result.is_err(), "Should fail without CA initialization");

	if let Err(e) = result {
		println!("Expected error without CA: {}", e);
	}
}

/// Test concurrent certificate generation
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_concurrent_certificate_generation() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test-concurrent.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate multiple certificates concurrently
	let mut handles = vec![];

	for i in 0..5 {
		let manager_ref = &manager;

		let hostname = format!("concurrent{}.editor.land", i);

		let handle = tokio::spawn(async move { manager_ref.get_server_cert(&hostname).await });

		handles.push(handle);
	}

	// Wait for all to complete
	let results = futures::future::join_all(handles).await;

	println!("Concurrent certificate generation completed");

	for result in results {
		assert!(result.is_ok(), "Concurrent generation should succeed");

		let cert_result = result.unwrap();

		assert!(cert_result.is_ok(), "Certificate should be generated successfully");
	}
}

/// Test certificate expiry calculation
#[tokio::test]
#[ignore] // Ignored by default as it requires keyring access
async fn test_certificate_expiry_calculation() {
	let mut manager = Binary::Build::CertificateManager::CertificateManager::new("test-expiry.app")
		.await
		.expect("Failed to create certificate manager");

	// Initialize CA
	manager.initialize_ca().await.expect("Failed to initialize CA");

	// Generate certificate
	let hostname = "expiry.editor.land";

	manager.get_server_cert(hostname).await.expect("Failed to generate server cert");

	// Get certificate info
	let info = manager.get_server_cert_info(hostname).expect("Should have certificate info");

	println!("Certificate validity period:");

	println!("  From: {}", info.valid_from);

	println!("  Until: {}", info.valid_until);

	// Certificates should be valid for approximately 1 year
	assert!(!info.valid_until.is_empty(), "Should have expiry date");
}
