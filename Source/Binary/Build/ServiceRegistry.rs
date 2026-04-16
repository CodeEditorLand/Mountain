//! # Service Registry Module
//!
//! ## RESPONSIBILITIES
//!
//! - Track mapping from land:// domain names to local HTTP service ports
//! - Provide thread-safe access using Arc<RwLock<>>
//! - Support service registration and lookup
//! - Enable health checks for registered services
//!
//! ## ARCHITECTURAL ROLE
//!
//! The ServiceRegistry provides the bridge between land:// URIs and local
//! services:
//!
//! ```text
//! land://code.editor.land/path ──► ServiceRegistry ──► http://127.0.0.1:PORT/path
//! ```
//!
//! ## THREAD SAFETY
//!
//! - Uses Arc<RwLock<HashMap<String, LocalService>>> for concurrent access
//! - Multiple readers allowed concurrently
//! - Writers lock exclusively
//!
//! ## USAGE
//!
//! ```rust
//! let registry = ServiceRegistry::new();
//! registry.register("code.editor.land".to_string(), 8080, Some("/health".to_string()));
//!
//! let service = registry.lookup("code.editor.land").unwrap();
//! assert_eq!(service.port, 8080);
//! ```

use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

#[allow(unused_imports)]
use http::{Request as HttpRequest, Response as HttpResponse, header};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};
use crate::dev_log;

/// Represents a local HTTP/HTTPS service registered with the land:// scheme
///
/// # Fields
///
/// - `name`: Domain name (e.g., "code.editor.land")
/// - `port`: Local port where the service is listening
/// - `tls_port`: Optional TLS port for HTTPS (defaults to port + 1000 if not
///   specified)
/// - `use_tls`: Whether the service uses HTTPS
/// - `health_check_path`: Optional path for health check endpoint
#[derive(Debug, Clone)]
pub struct LocalService {
	pub name:String,
	pub port:u16,
	pub tls_port:Option<u16>,
	pub use_tls:bool,
	pub health_check_path:Option<String>,
}

impl LocalService {
	/// Get the appropriate port based on TLS configuration
	pub fn get_port(&self) -> u16 {
		if self.use_tls {
			self.tls_port.unwrap_or_else(|| self.port + 1000)
		} else {
			self.port
		}
	}
}

/// Registry for tracking local HTTP/HTTPS services
///
/// Provides thread-safe methods to register and lookup services by domain name.
/// Supports both HTTP and HTTPS protocols with automatic TLS certificate
/// provisioning.
#[derive(Clone)]
pub struct ServiceRegistry {
	/// Inner storage using `Arc<RwLock>` for thread-safe concurrent access
	services:Arc<RwLock<HashMap<String, LocalService>>>,
	/// Optional certificate manager for HTTPS support
	cert_manager:Option<std::sync::Arc<std::sync::Mutex<super::CertificateManager::CertificateManager>>>,
}

impl ServiceRegistry {
	/// Create a new ServiceRegistry instance
	///
	/// Returns an empty registry ready to accept service registrations.
	pub fn new() -> Self {
		dev_log!("lifecycle", "[ServiceRegistry] Creating new ServiceRegistry");
		Self { services:Arc::new(RwLock::new(HashMap::new())), cert_manager:None }
	}

	/// Create a new ServiceRegistry instance with TLS support
	///
	/// # Parameters
	///
	/// * `cert_manager` - Certificate manager for provisioning TLS certificates
	///
	/// Returns a registry ready to accept both HTTP and HTTPS service
	/// registrations.
	pub fn with_tls(
		cert_manager:std::sync::Arc<std::sync::Mutex<super::CertificateManager::CertificateManager>>,
	) -> Self {
		dev_log!("lifecycle", "[ServiceRegistry] Creating new ServiceRegistry with TLS support");
		Self { services:Arc::new(RwLock::new(HashMap::new())), cert_manager:Some(cert_manager) }
	}

	/// Register a local HTTP service
	///
	/// # Parameters
	///
	/// - `name`: Domain name (e.g., "code.editor.land")
	/// - `port`: Local port where the service is listening
	/// - `health_check_path`: Optional path for health check endpoint (e.g.,
	///   "/health")
	///
	/// # Example
	///
	/// ```rust
	/// registry.register("code.editor.land".to_string(), 8080, Some("/health".to_string())); 
	/// ```
	pub fn register(&self, name:String, port:u16, health_check_path:Option<String>) {
		self.register_with_options(name, port, None, false, health_check_path);
	}

	/// Register a local service with TLS options
	///
	/// # Parameters
	///
	/// - `name`: Domain name (e.g., "code.editor.land")
	/// - `port`: Local HTTP port
	/// - `tls_port`: Optional TLS port (defaults to port + 1000)
	/// - `use_tls`: Whether to enable HTTPS
	/// - `health_check_path`: Optional path for health check endpoint
	///
	/// # Example
	///
	/// ```rust
	/// // Register with TLS enabled
	/// registry.register_with_options(
	/// 	"code.editor.land".to_string(),
	/// 	8080,
	/// 	None, // Use default TLS port (9080)
	/// 	true,
	/// 	Some("/health".to_string()),
	/// );
	/// ```
	pub fn register_with_options(
		&self,
		name:String,
		port:u16,
		tls_port:Option<u16>,
		use_tls:bool,
		health_check_path:Option<String>,
	) {
		dev_log!("lifecycle", 
			"[ServiceRegistry] Registering service: {} -> HTTP:{}, TLS:{}, use_tls:{}",
			name,
			port,
			tls_port.unwrap_or(port + 1000),
			use_tls
		);

		let service = LocalService { name:name.clone(), port, tls_port, use_tls, health_check_path };

		// Pre-provision TLS certificate if needed
		if use_tls {
			if let Some(cert_manager) = &self.cert_manager {
				// NOTE: TLS certificate is generated on-demand when needed
				dev_log!("lifecycle", "[ServiceRegistry] TLS will be provisioned on-demand for {}", name);
			} else {
				dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} requested TLS but no certificate manager available",
					name);
			}
		}

		if let Ok(mut services) = self.services.write() {
			// Check if service already exists
			if services.contains_key(&name) {
				dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} already registered, overwriting", name);
			}
			services.insert(name.clone(), service);
			dev_log!("lifecycle", "[ServiceRegistry] Service {} registered successfully", name);
		} else {
			dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire write lock for registration");
		}
	}

	/// Look up a service by domain name
	///
	/// # Parameters
	///
	/// - `name`: Domain name to look up
	///
	/// # Returns
	///
	/// - `Some(LocalService)` if found
	/// - `None` if not registered
	///
	/// # Example
	///
	/// ```rust
	/// let service = registry.lookup("code.editor.land");
	/// if let Some(svc) = service {
	/// 	println!("Service running on port {}", svc.port);
	/// }
	/// ```
	pub fn lookup(&self, name:&str) -> Option<LocalService> {
		dev_log!("lifecycle", "[ServiceRegistry] Looking up service: {}", name);

		if let Ok(services) = self.services.read() {
			let service = services.get(name).cloned();
			if service.is_some() {
				dev_log!("lifecycle", "[ServiceRegistry] Service {} found", name);
			} else {
				dev_log!("lifecycle", "[ServiceRegistry] Service {} not found", name);
			}
			service
		} else {
			dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire read lock for lookup");
			None
		}
	}

	/// Get all registered services
	///
	/// # Returns
	///
	/// A vector of all registered LocalService instances
	pub fn all_services(&self) -> Vec<LocalService> {
		if let Ok(services) = self.services.read() {
			services.values().cloned().collect()
		} else {
			dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire read lock for all_services");
			Vec::new()
		}
	}

	/// Perform a health check on a registered service
	///
	/// # Parameters
	///
	/// - `name`: Domain name of the service to check
	///
	/// # Returns
	///
	/// - `Ok(true)` if service is healthy and responding
	/// - `Ok(false)` if service is not healthy
	/// - `Err` if service not found or health check fails
	pub async fn health_check(&self, name:&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
		let service = self.lookup(name).ok_or_else(|| format!("Service {} not found", name))?;

		let health_path = service.health_check_path.as_deref().unwrap_or("/health");
		let addr = format!("127.0.0.1:{}", service.port);

		dev_log!("lifecycle", 
			"[ServiceRegistry] Performing health check for {} at {}:{}",
			name, addr, health_path
		);

		// Try to connect to the service
		match TcpStream::connect(&addr).await {
			Ok(mut stream) => {
				// Send simple HTTP GET request
				let request = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n", health_path, service.port);

				match stream.write_all(request.as_bytes()).await {
					Ok(_) => {
						// Try to read response
						let mut buffer = [0u8; 1024];
						match stream.read(&mut buffer).await {
							Ok(n) => {
								let response = String::from_utf8_lossy(&buffer[..n]);
								let is_healthy = response.contains("HTTP/1.1 200") || response.contains("HTTP/1.0 200");
								if is_healthy {
									dev_log!("lifecycle", "[ServiceRegistry] Service {} is healthy", name);
								} else {
									dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} health check failed: not 200", name);
								}
								Ok(is_healthy)
							},
							Err(e) => {
								dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} health check failed to read: {}", name, e);
								Ok(false)
							},
						}
					},
					Err(e) => {
						dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} health check failed to write: {}", name, e);
						Ok(false)
					},
				}
			},
			Err(e) => {
				dev_log!("lifecycle", "warn: [ServiceRegistry] Service {} health check failed to connect: {}", name, e);
				Ok(false)
			},
		}
	}

	/// Remove a service from the registry
	///
	/// # Parameters
	///
	/// - `name`: Domain name of the service to remove
	///
	/// # Returns
	///
	/// - `Some(LocalService)` if service was removed
	/// - `None` if service was not found
	pub fn unregister(&self, name:&str) -> Option<LocalService> {
		dev_log!("lifecycle", "[ServiceRegistry] Unregistering service: {}", name);

		if let Ok(mut services) = self.services.write() {
			services.remove(name)
		} else {
			dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire write lock for unregistration");
			None
		}
	}

	/// Get TLS configuration for a service (if available)
	///
	/// # Parameters
	///
	/// - `name`: Domain name of the service
	///
	/// # Returns
	///
	/// - `Some(Arc<ServerConfig>)` if service uses TLS and certificate manager
	///   is available
	/// - `None` if service doesn't use TLS or certificate manager is not
	///   configured
	pub async fn get_tls_config(&self, name:&str) -> Option<std::sync::Arc<rustls::ServerConfig>> {
		let service = self.lookup(name)?;

		if !service.use_tls {
			return None;
		}

		let cert_manager = self.cert_manager.as_ref()?;
		let manager = cert_manager
			.lock()
			.map_err(|e| {
				dev_log!("lifecycle", "error: [ServiceRegistry] Failed to acquire lock: {}", e);
			})
			.ok()?;
		manager.build_server_config(name).await.ok()
	}

	/// Check if a service uses TLS
	///
	/// # Parameters
	///
	/// - `name`: Domain name of the service
	///
	/// # Returns
	///
	/// `true` if the service is configured to use TLS, `false` otherwise
	pub fn uses_tls(&self, name:&str) -> bool { self.lookup(name).map(|s| s.use_tls).unwrap_or(false) }
}

impl Default for ServiceRegistry {
	fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_register_and_lookup() {
		let registry = ServiceRegistry::new();

		registry.register("test.service.land".to_string(), 8080, Some("/health".to_string()));

		let service = registry.lookup("test.service.land").unwrap();
		assert_eq!(service.name, "test.service.land");
		assert_eq!(service.port, 8080);
		assert_eq!(service.health_check_path, Some("/health".to_string()));
	}

	#[test]
	fn test_lookup_nonexistent() {
		let registry = ServiceRegistry::new();

		let service = registry.lookup("nonexistent.service.land");
		assert!(service.is_none());
	}

	#[test]
	fn test_all_services() {
		let registry = ServiceRegistry::new();

		registry.register("service1.land".to_string(), 8080, None);
		registry.register("service2.land".to_string(), 8081, None);

		let services = registry.all_services();
		assert_eq!(services.len(), 2);
	}

	#[test]
	fn test_unregister() {
		let registry = ServiceRegistry::new();

		registry.register("test.service.land".to_string(), 8080, None);
		assert!(registry.lookup("test.service.land").is_some());

		registry.unregister("test.service.land");
		assert!(registry.lookup("test.service.land").is_none());
	}

	#[test]
	fn test_overwrite_registration() {
		let registry = ServiceRegistry::new();

		registry.register("test.service.land".to_string(), 8080, None);
		registry.register("test.service.land".to_string(), 9090, None);

		let service = registry.lookup("test.service.land").unwrap();
		assert_eq!(service.port, 9090);
	}

	#[test]
	fn test_tls_service() {
		let registry = ServiceRegistry::new();

		registry.register_with_options(
			"secure.service.land".to_string(),
			8080,
			Some(8443),
			true,
			Some("/health".to_string()),
		);

		let service = registry.lookup("secure.service.land").unwrap();
		assert_eq!(service.name, "secure.service.land");
		assert_eq!(service.port, 8080);
		assert_eq!(service.tls_port, Some(8443));
		assert_eq!(service.use_tls, true);
		assert_eq!(service.get_port(), 8443);
	}

	#[test]
	fn test_default_tls_port() {
		let registry = ServiceRegistry::new();

		registry.register_with_options(
			"secure.service.land".to_string(),
			8080,
			None, // Use default TLS port (8080 + 1000 = 9080)
			true,
			None,
		);

		let service = registry.lookup("secure.service.land").unwrap();
		assert_eq!(service.tls_port, None); // Explicitly None
		assert_eq!(service.get_port(), 9080); // But get_port() returns default
	}
}
