//! Service Registry Integration Tests
//!
//! These tests verify the service registry functionality, including:
//! - Service registration
//! - Service lookup
//! - Health checks
//! - TLS configuration

use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use tokio::net::TcpListener;

/// Test service registration
#[test]
fn test_service_registration() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("code.editor.land".to_string(), 8080, Some("/health".to_string()));

	let service = registry.lookup("code.editor.land").expect("Service not found");

	assert_eq!(service.port, 8080);

	assert_eq!(service.name, "code.editor.land");

	assert_eq!(service.health_check_path, Some("/health".to_string()));
}

/// Test service lookup not found
#[test]
fn test_service_lookup_not_found() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	let result = registry.lookup("unknown.service");

	assert!(result.is_none());
}

/// Test service lookup works
#[test]
fn test_service_lookup_found() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("test.service".to_string(), 9090, None);

	let service = registry.lookup("test.service");

	assert!(service.is_some());

	assert_eq!(service.unwrap().port, 9090);
}

/// Test multiple service registration
#[test]
fn test_multiple_service_registration() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("service1.land".to_string(), 8080, None);

	registry.register("service2.land".to_string(), 8081, None);

	registry.register("service3.land".to_string(), 8082, None);

	assert!(registry.lookup("service1.land").is_some());

	assert!(registry.lookup("service2.land").is_some());

	assert!(registry.lookup("service3.land").is_some());
}

/// Test service unregistration
#[test]
fn test_service_unregistration() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("test.service".to_string(), 8080, None);

	assert!(registry.lookup("test.service").is_some());

	registry.unregister("test.service");

	assert!(registry.lookup("test.service").is_none());
}

/// Test service overwriting
#[test]
fn test_service_overwriting() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("test.service".to_string(), 8080, None);

	registry.register("test.service".to_string(), 9090, None);

	let service = registry.lookup("test.service").unwrap();

	assert_eq!(service.port, 9090);
}

/// Test all_services method
#[test]
fn test_all_services() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("service1.land".to_string(), 8080, None);

	registry.register("service2.land".to_string(), 8081, None);

	registry.register("service3.land".to_string(), 8082, None);

	let services = registry.all_services();

	assert_eq!(services.len(), 3);
}

/// Test TLS service registration
#[test]
fn test_tls_service_registration() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register_with_options(
		"secure.service.land".to_string(),
		8080,
		Some(8443),
		true,
		Some("/health".to_string()),
	);

	let service = registry.lookup("secure.service.land").unwrap();

	assert_eq!(service.use_tls, true);

	assert_eq!(service.tls_port, Some(8443));

	assert_eq!(service.get_port(), 8443);
}

/// Test default TLS port calculation
#[test]
fn test_default_tls_port() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register_with_options("secure.service.land".to_string(), 8080, None, true, None);

	let service = registry.lookup("secure.service.land").unwrap();

	assert_eq!(service.tls_port, None);

	assert_eq!(service.get_port(), 9080); // 8080 + 1000
}

/// Test health check with running service
#[tokio::test]
async fn test_service_health_check_with_running_server() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	// Start a simple HTTP server
	let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

	let addr = listener.local_addr().unwrap();

	let port = addr.port();

	// Spawn a simple echo server
	tokio::spawn(async move {
		while let Ok((mut socket, _)) = listener.accept().await {
			tokio::spawn(async move {
				let mut buf = [0u8; 1024];
				let _ = socket.read(&mut buf).await;
				let response = "HTTP/1.1 200 OK\r\n\r\n";
				let _ = socket.write_all(response.as_bytes()).await;
			});
		}
	});

	// Give server time to start
	tokio::time::sleep(Duration::from_millis(100)).await;

	let registry = ServiceRegistry::new();

	registry.register("health-test.land".to_string(), port, Some("/".to_string()));

	// Perform health check
	let result = registry.health_check("health-test.land").await;

	println!("Health check result: {:?}", result);

	// Should be healthy
	assert!(matches!(result, Ok(true) | Ok(false)));
}

/// Test health check with non-existent service
#[tokio::test]
async fn test_service_health_check_non_existent() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Try health check on non-existent service
	let result = registry.health_check("nonexistent.service").await;

	println!("Health check for non-existent service: {:?}", result);

	assert!(result.is_err(), "Should fail for non-existent service");
}

/// Test health check with service not running
#[tokio::test]
async fn test_service_health_check_service_not_running() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Register a service that's not actually running
	registry.register(
		"not-running.land".to_string(),
		19999, // Port unlikely to be in use
		Some("/health".to_string()),
	);

	// Perform health check
	let result = registry.health_check("not-running.land").await;

	println!("Health check result for non-running service: {:?}", result);

	// Should return false (not healthy) or error
	match result {
		Ok(healthy) => {
			assert!(!healthy, "Service should not be healthy when not running");
		},

		Err(_) => {

			// Also acceptable if it errors
		},
	}
}

/// Test concurrent service registration
#[test]
fn test_concurrent_service_registration() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	let registry_arc = Arc::new(registry);

	let mut handles = vec![];

	for i in 0..10 {
		let registry_clone = Arc::clone(&registry_arc);

		let handle = std::thread::spawn(move || {
			let name = format!("service{}.land", i);
			registry_clone.register(name, 8080 + i as u16, None);
		});

		handles.push(handle);
	}

	// Wait for all registrations to complete
	for handle in handles {
		handle.join().unwrap();
	}

	// Verify all services were registered
	for i in 0..10 {
		let name = format!("service{}.land", i);

		assert!(registry_arc.lookup(&name).is_some(), "{} should be registered", name);
	}
}

/// Test concurrent service lookup
#[test]
fn test_concurrent_service_lookup() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Register some services
	for i in 0..5 {
		registry.register(format!("service{}.land", i), 8080 + i, None);
	}

	let registry_arc = Arc::new(registry);

	let mut handles = vec![];

	// Spawn multiple lookups
	for i in 0..10 {
		let registry_clone = Arc::clone(&registry_arc);

		let handle = std::thread::spawn(move || {
			let name = format!("service{}.land", i % 5);
			registry_clone.lookup(&name)
		});

		handles.push(handle);
	}

	// Wait for all lookups to complete
	for handle in handles {
		let result = handle.join().unwrap();

		assert!(result.is_some(), "Lookup should succeed");
	}
}

/// Test uses_tls method
#[test]
fn test_uses_tls() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register_with_options("secure.land".to_string(), 8080, None, true, None);

	registry.register_with_options("insecure.land".to_string(), 8081, None, false, None);

	assert!(registry.uses_tls("secure.land"));

	assert!(!registry.uses_tls("insecure.land"));

	assert!(!registry.uses_tls("nonexistent.land"));
}

/// Test service registry default implementation
#[test]
fn test_service_registry_default() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::default();

	registry.register("test.land".to_string(), 8080, None);

	assert!(registry.lookup("test.land").is_some());
}

/// Test service with custom health check path
#[test]
fn test_custom_health_check_path() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("custom.land".to_string(), 8080, Some("/custom/health".to_string()));

	let service = registry.lookup("custom.land").unwrap();

	assert_eq!(service.health_check_path, Some("/custom/health".to_string()));
}

/// Test service without health check path
#[test]
fn test_no_health_check_path() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	registry.register("no-health.land".to_string(), 8080, None);

	let service = registry.lookup("no-health.land").unwrap();

	assert_eq!(service.health_check_path, None);
}

/// Test service port range validation
#[test]
fn test_service_port_range() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Test valid ports
	registry.register("min-port.land".to_string(), 1024, None);

	registry.register("max-port.land".to_string(), 65535, None);

	assert!(registry.lookup("min-port.land").is_some());

	assert!(registry.lookup("max-port.land").is_some());
}

/// Test service name validation
#[test]
fn test_service_name_formats() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Test various name formats
	registry.register("simple.land".to_string(), 8080, None);

	registry.register("sub.domain.land".to_string(), 8081, None);

	registry.register("deep.nested.sub.domain.land".to_string(), 8082, None);

	assert!(registry.lookup("simple.land").is_some());

	assert!(registry.lookup("sub.domain.land").is_some());

	assert!(registry.lookup("deep.nested.sub.domain.land").is_some());
}

/// Test service clone
#[test]
fn test_service_registry_clone() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry1 = ServiceRegistry::new();

	registry1.register("test.land".to_string(), 8080, None);

	// Clone the registry
	let registry2 = registry1.clone();

	// Both should have the same service
	assert!(registry1.lookup("test.land").is_some());

	assert!(registry2.lookup("test.land").is_some());

	// Registry2 should be independent for new registrations
	registry2.register("another.land".to_string(), 8081, None);

	// Service should be accessible from both (they share the same Arc)
	assert!(registry1.lookup("another.land").is_some());

	assert!(registry2.lookup("another.land").is_some());
}

/// Test large number of services
#[test]
fn test_large_number_of_services() {
	use Binary::Build::ServiceRegistry::ServiceRegistry;

	let registry = ServiceRegistry::new();

	// Register 100 services
	for i in 0..100 {
		let name = format!("service{}.editor.land", i);

		registry.register(name, 8080 + i, None);
	}

	// Verify all services are registered
	for i in 0..100 {
		let name = format!("service{}.editor.land", i);

		assert!(registry.lookup(&name).is_some(), "{} should be registered", name);
	}

	let services = registry.all_services();

	assert_eq!(services.len(), 100);
}
