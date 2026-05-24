pub mod GetPort;
pub mod New;
pub mod WithTls;
pub mod Register;
pub mod RegisterWithOptions;
pub mod Lookup;
pub mod AllServices;
pub mod HealthCheck;
pub mod Unregister;
pub mod GetTlsConfig;
pub mod UsesTls;

use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use http::{Request as HttpRequest, Response as HttpResponse, header};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};
use crate::dev_log;

/// Represents a local HTTP/HTTPS service registered with the land:// scheme
/// # Fields
/// - `name`: Domain name (e.g., "code.land.playform.cloud")
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

/// Registry for tracking local HTTP/HTTPS services
/// Provides thread-safe methods to register and lookup services by domain name.
/// Supports both HTTP and HTTPS protocols with automatic TLS certificate
/// provisioning.
#[derive(Clone)]
pub struct Struct {
	/// Inner storage using `Arc<RwLock>` for thread-safe concurrent access
	services:Arc<RwLock<HashMap<String, LocalService>>>,

	/// Optional certificate manager for HTTPS support
	cert_manager:Option<std::sync::Arc<std::sync::Mutex<super::CertificateManager::CertificateManager>>>,
}
}
