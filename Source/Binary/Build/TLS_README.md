# TLS Certificate Management Infrastructure

## Overview

This module provides a comprehensive TLS certificate management system for HTTPS
services in the CodeEditorLand application. It manages a root CA certificate and
automatically generates server certificates signed by the CA, enabling secure
HTTPS connections for local development and testing.

## Architecture

### Certificate Hierarchy

```
Root CA Certificate (stored in OS keyring)
└── ECDSA P-256 private key (secure storage)
    └── Server Certificates (cached per hostname)
        ├── code.land.playform.cloud
        ├── api.land.playform.cloud
        └── ...other services
            ├── Certificate chain (server cert + CA cert)
            └── ECDSA P-256 private key
```

### Key Components

1. **CertificateManager** - Core module for certificate lifecycle management
2. **ServiceRegistry** - Integration point for HTTPS service registration

## Trust Model

### Root CA Certificate

- **Storage**: OS keyring (platform-specific secure credential manager)
- **Algorithm**: ECDSA P-256 (matching DNSSEC key algorithm)
- **Validity**: 10 years
- **Usage**: Certificate signing only (CA:TRUE, keyCertSign, CRLSign)

### Server Certificates

- **Generation**: Automatic when service is registered
- **Signature**: Signed by root CA
- **Algorithm**: ECDSA P-256 for consistency
- **Validity**: 1 year (auto-renewable)
- **Subject Alternative Names**:
    - DNS: hostname (e.g., "code.land.playform.cloud")
    - IP: 127.0.0.1 (IPv4 localhost)
    - IP: ::1 (IPv6 localhost)

### Trust Establishment

The webview must trust the root CA certificate to validate server certificates:

1. **Option 1 - System Trust Store**: Install CA certificate in OS trust store
2. **Option 2 - Custom Validation**: Configure webview to accept the CA
   certificate
3. **Option 3 - Certificate Pinning**: Pin server certificates by fingerprint

## Keyring Storage

### Platform-Specific Locations

| Platform | Storage Location                              |
| -------- | --------------------------------------------- |
| macOS    | Keychain Access                               |
| Linux    | D-Bus Secret Service (gnome-keyring, KWallet) |
| Windows  | Windows Credential Manager                    |

### Storage Structure

```
Service: "CodeEditorLand-TLS"
Entries:
  - "<app_id>:ca_certificate"    -> CA certificate PEM
  - "<app_id>:ca_private_key"    -> CA private key PEM
```

### Security Properties

- Private keys are never stored in plaintext files
- Keys are encrypted by the OS credential manager
- Access is controlled by OS security policies
- Keys are never logged or exposed in diagnostic output

## Certificate Lifecycle

### 1. Initialization

```rust
use Binary::Build::CertificateManager::CertificateManager;

// Create certificate manager
let mut cert_manager = CertificateManager::new("myapp").await?;

// Initialize or load CA
cert_manager.initialize_ca().await?;
```

**Process**:

1. Check keyring for existing CA certificate
2. If found, load and parse signing key
3. If not found, generate new self-signed CA
4. Store CA certificate and private key in keyring

### 2. Service Registration

```rust
use Binary::Build::ServiceRegistry::ServiceRegistry;

// Create registry with TLS support
let registry = ServiceRegistry::with_tls(cert_manager.clone());

// Register HTTPS service
registry.register_with_options(
    "code.land.playform.cloud".to_string(),
    8080,   // HTTP port
    Some(8443),  // TLS port
    true,   // Enable TLS
    Some("/health".to_string())
);
```

**Process**:

1. Register service in registry
2. Auto-provision TLS certificate (background task)
3. Generate server key pair
4. Sign with CA
5. Cache certificate in memory

### 3. Certificate Usage

```rust
// Get TLS configuration for HTTPS server
let server_config = cert_manager
    .build_server_config("code.land.playform.cloud")
    .await?;

// Use with rustls-based server
let listener = tokio_rustls::TlsListener::bind(addr, server_config).await?;
```

### 4. Certificate Renewal

```rust
// Check if renewal is needed
if cert_manager.should_renew(&cert_pem) {
    // Force renewal
    cert_manager.renew_certificate("code.land.playform.cloud").await?;
}
```

**Renewal Triggers**:

- Certificate expiring within 30 days
- Manual renewal requested
- Service re-registration

## Tauri Commands

### Available Commands

| Command                    | Description                    | Parameters         | Returns                            |
| -------------------------- | ------------------------------ | ------------------ | ---------------------------------- |
| `tls_initialize`           | Initialize certificate manager | None               | Status message                     |
| `tls_get_ca_cert`          | Get CA certificate PEM         | None               | CA certificate string              |
| `tls_get_server_cert_info` | Get server certificate info    | `hostname: String` | `CertificateInfo`                  |
| `tls_renew_certificate`    | Force certificate renewal      | `hostname: String` | Status message                     |
| `tls_get_all_certs`        | List all certificates          | None               | `HashMap<String, CertificateInfo>` |
| `tls_check_cert_status`    | Check renewal status           | `hostname: String` | `CertificateStatus`                |
| `tls_generate_cert`        | Generate certificate           | `hostname: String` | `CertificateGenerationResult`      |
| `tls_delete_cert`          | Delete certificate             | `hostname: String` | Status message                     |

### Usage Examples

```typescript
// Initialize TLS
await invoke("tls_initialize");

// Get CA certificate for trust store installation
const caCert = await invoke("tls_get_ca_cert");
console.log("CA Certificate:", caCert);

// Get server certificate info
const certInfo = await invoke("tls_get_server_cert_info", {
	hostname: "code.land.playform.cloud",
});
console.log("Valid until:", certInfo.valid_until);

// Check certificate status
const status = await invoke("tls_check_cert_status", {
	hostname: "code.land.playform.cloud",
});
if (status.needs_renewal) {
	console.log("Certificate needs renewal");
	await invoke("tls_renew_certificate", {
		hostname: "code.land.playform.cloud",
	});
}

// List all certificates
const allCerts = await invoke("tls_get_all_certs");
console.log("All certificates:", allCerts);
```

## Integration with Existing Services

### Updating ServiceRegistry

The
[`ServiceRegistry`](https://github.com/CodeEditorLand/Mountain/tree/Current/Source/Binary/Build/ServiceRegistry.rs)
has been updated to support HTTPS services:

```rust
// Traditional HTTP-only service (backward compatible)
registry.register("service.land".to_string(), 8080, Some("/health"));

// New HTTPS-enabled service
registry.register_with_options(
    "secure.service.land".to_string(),
    8080,   // HTTP port
    Some(8443),  // TLS port (or use default port + 1000)
    true,   // Enable TLS
    Some("/health".to_string())
);
```

### HTTP vs HTTPS Routing

The
[`Scheme`](https://github.com/CodeEditorLand/Mountain/tree/Current/Source/Binary/Build/Scheme.rs)
handler can be extended to support HTTPS:

```rust
// Check if service uses TLS
if registry.uses_tls(&domain) {
    // Use HTTPS with TLS configuration
    let tls_config = registry.get_tls_config(&domain).await?;
    // Connect to HTTPS service
} else {
    // Use HTTP (existing behavior)
}
```

## Security Considerations

### Private Key Protection

- Private keys are stored in OS keyring, not in files
- Keys are never logged or exposed in diagnostics
- Memory is zeroed after use where possible
- Keyring access is protected by OS security

### Certificate Validation

- Server certificates include proper Subject Alternative
  Names -证书链包括服务器证书和CA证书
- Certificates use secure elliptic curve (P-256)
- Automatic renewal prevents expired certificates

### Best Practices

1. **Rotate CA Private Key**: If CA private key is compromised, delete from
   keyring and regenerate
2. **Monitor Certificate Expiry**: Implement monitoring for certificates nearing
   expiry
3. **Secure CA Distribution**: Only distribute CA certificate to trusted
   entities
4. **Use Strong Entropy**: Use OS-provided RNG for key generation (already
   implemented)
5. **Update Dependencies**: Keep cryptographic libraries updated

## Troubleshooting

### "CA not initialized" Error

**Cause**: Certificate manager not initialized before use.

**Solution**:

```rust
let mut cert_manager = CertificateManager::new("app").await?;
cert_manager.initialize_ca().await?;
```

### "Failed to load CA from keyring" Error

**Cause**: CA certificate deleted from keyring or corrupted.

**Solution**: Clear keyring entry and reinitialize:

```rust
// CA will be regenerated on next initialize_ca()
cert_manager.initialize_ca().await?;
```

### "Certificate validation failed" Error

**Cause**: CA certificate not trusted by webview.

**Solution**: Install CA certificate in system trust store or configure webview:

```typescript
const caCert = await invoke("tls_get_ca_cert");
// Install CA cert in OS trust store
```

### "Service not found" Lookup Error

**Cause**: Service not registered in ServiceRegistry.

**Solution**: Register service first:

```rust
registry.register_with_options(
    "service.land".to_string(),
    8080,
    Some(8443),
    true,
    None
);
```

## Performance Considerations

### Certificate Caching

- Server certificates are cached in memory after generation
- Certificates are retrieved from cache on subsequent uses
- Avoids expensive signature operations for each request

### Async Operations

- Certificate generation is async (non-blocking)
- Background tasks for certificate provisioning
- Does not block main application thread

### Memory Usage

- Certificate cache is bounded by number of registered services
- Each certificate ~1-2 KB in memory
- Typical use: < 100 KB for dozens of services

## Testing

### Unit Tests

Run unit tests for CertificateManager:

```bash
cargo test -p Mountain --lib CertificateManager
```

Run unit tests for ServiceRegistry TLS integration:

```bash
cargo test -p Mountain --lib ServiceRegistry::test_tls_service
```

### Integration Tests

Test certificate generation and validation:

```rust
#[tokio::test]
async fn test_full_certificate_lifecycle() {
    let mut cert_manager = CertificateManager::new("test").await.unwrap();
    cert_manager.initialize_ca().await.unwrap();

    let server_config = cert_manager.get_server_cert("test.local").await.unwrap();
    assert!(server_config.cert_chain.len() > 0);

    let cert_info = cert_manager.get_server_cert_info("test.local").unwrap();
    assert_eq!(cert_info.subject, "CN=test.local");
}
```

## Dependencies

### Runtime Dependencies

- `rustls` - TLS library (pure Rust, no OpenSSL)
- `p256` - ECDSA P-256 elliptic curve operations
- `rcgen` - Certificate generation
- `x509-parser` - X.509 certificate parsing
- `pem` - PEM encoding/decoding
- `keyring` - OS keyring access
- `chrono` - Date/time handling
- `tokio` - Async runtime

### All dependencies are already in workspace:

```toml
rustls = { workspace = true }
keyring = { workspace = true }
chrono = { workspace = true }
tokio = { workspace = true }
```

Added for TLS support:

```toml
pem = "3"
rcgen = "0.13"
p256 = "0.13"
x509-parser = "0.16"
rustls-pki-types = "1"
```

## Future Enhancements

### Potential Improvements

1. **Certificate Revocation**: Implement CRL (Certificate Revocation List)
   support
2. **OCSP Stapling**: Online Certificate Status Protocol for real-time
   validation
3. **Multiple CA Support**: Support for multiple trusted CAs
4. **Certificate Export**: Export certificates in various formats (DER, JKS,
   PKCS12)
5. **Health Checks**: Periodic certificate validity checks with notifications
6. **Metric Collection**: Track certificate generation/renewal metrics
7. **Backup/Restore**: Export/import CA certificate for backup
8. **Certificate Pinning**: Support for certificate fingerprint pinning

### Webview Integration

Custom certificate validation for Tauri webview:

```rust
use tauri::webview::WebviewBuilder;

// Configure webview to trust our CA
webview = webview.builder()
    .on_web_resource_request(|request, responder| {
        // Custom certificate validation
        // Verify certificate chain against our CA
    });
```

## References

- **rustls**: https://docs.rs/rustls/
- **RCGen**: https://docs.rs/rcgen/
- **X.509**: https://www.rfc-editor.org/rfc/rfc5280
- **ECDSA P-256**: NIST FIPS 186-4
- **Keyring**: https://docs.rs/keyring/

## License

This module is part of the CodeEditorLand project and follows the project
license.

## Support

For issues or questions:

1. Check the troubleshooting section
2. Review the inline documentation
3. Check the test files for usage examples
4. Open an issue in the project repository
