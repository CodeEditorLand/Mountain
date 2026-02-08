//! # Service Discovery and Information
//!
//! Provides service discovery and information tracking for Mountain services.
//! Used to monitor and manage registered services.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// State of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is running normally
    Running,
    /// Service is degraded but operational
    Degraded,
    /// Service is stopped
    Stopped,
    /// Service has encountered an error
    Error,
    /// Service is starting up
    Starting,
    /// Service is shutting down
    ShuttingDown,
}

impl ServiceState {
    /// Check if service is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, ServiceState::Running | ServiceState::Degraded | ServiceState::Starting)
    }
}

/// Information about a single service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Current state
    pub state: ServiceState,
    /// When the service entered its current state
    pub state_since: Instant,
    /// Service uptime
    pub uptime: Duration,
    /// Last heartbeat timestamp
    pub last_heartbeat: Option<Instant>,
    /// Services this service depends on
    pub dependencies: Vec<String>,
    /// Performance metrics for this service
    pub performance: ServicePerformance,
    /// Optional network endpoint
    pub endpoint: Option<ServiceEndpoint>,
}

/// Performance metrics for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformance {
    /// Request count
    pub request_count: u64,
    /// Error count
    pub error_count: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Last updated timestamp
    pub last_updated: Instant,
}

impl ServicePerformance {
    /// Create new service performance metrics
    pub fn new() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&mut self, response_time_ms: f64) {
        self.request_count += 1;
        
        // Update average response time
        if self.average_response_time_ms == 0.0 {
            self.average_response_time_ms = response_time_ms;
        } else {
            self.average_response_time_ms = (self.average_response_time_ms * (self.request_count - 1) as f64
                + response_time_ms) / self.request_count as f64;
        }
        
        self.last_updated = Instant::now();
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
        self.last_updated = Instant::now();
    }

    /// Calculate error rate (0.0 to 1.0)
    pub fn error_rate(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.error_count as f64 / self.request_count as f64
    }
}

impl Default for ServicePerformance {
    fn default() -> Self {
        Self::new()
    }
}

/// Network endpoint for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Protocol (e.g., "ipc", "tcp", "udp")
    pub protocol: String,
    /// Host address
    pub address: String,
    /// Port number
    pub port: u16,
    /// Path (for Unix domain sockets)
    pub path: Option<String>,
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    pub fn new(protocol: impl Into<String>, address: impl Into<String>, port: u16) -> Self {
        Self {
            protocol: protocol.into(),
            address: address.into(),
            port,
            path: None,
        }
    }

    /// Create a Unix domain socket endpoint
    pub fn new_unix(path: impl Into<String>) -> Self {
        Self {
            protocol: "unix".to_string(),
            address: String::new(),
            port: 0,
            path: Some(path.into()),
        }
    }
}

impl ServiceInfo {
    /// Create a new service info
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            state: ServiceState::Starting,
            state_since: Instant::now(),
            uptime: Duration::ZERO,
            last_heartbeat: None,
            dependencies: Vec::new(),
            performance: ServicePerformance::new(),
            endpoint: None,
        }
    }

    /// Update service state
    pub fn update_state(&mut self, new_state: ServiceState) {
        self.state = new_state;
        self.state_since = Instant::now();
    }

    /// Record heartbeat
    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        
        // Update uptime if service is running
        if self.state == ServiceState::Running {
            self.uptime = self.state_since.elapsed();
        }
    }

    /// Check if service is healthy
    pub fn is_healthy(&self) -> bool {
        if !self.state.is_operational() {
            return false;
        }

        // Check if heartbeat is recent (within 30 seconds)
        if let Some(heartbeat) = self.last_heartbeat {
            if heartbeat.elapsed() > Duration::from_secs(30) {
                return false;
            }
        }

        // Check error rate (should be below 10%)
        if self.performance.error_rate() > 0.1 {
            return false;
        }

        true
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dependency: impl Into<String>) {
        self.dependencies.push(dependency.into());
    }
}

/// Registry of all discovered services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistry {
    /// Map of service name to service info
    pub services: HashMap<String, ServiceInfo>,
    /// Last discovery timestamp
    pub last_discovery: Instant,
    /// Configurable discovery interval
    pub discovery_interval: Duration,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(discovery_interval: Duration) -> Self {
        Self {
            services: HashMap::new(),
            last_discovery: Instant::now(),
            discovery_interval,
        }
    }

    /// Register a service
    pub fn register(&mut self, service: ServiceInfo) {
        self.services.insert(service.name.clone(), service);
        self.last_discovery = Instant::now();
    }

    /// Unregister a service
    pub fn unregister(&mut self, name: &str) -> Option<ServiceInfo> {
        self.services.remove(name).map(|service| {
            self.last_discovery = Instant::now();
            service
        })
    }

    /// Get service info by name
    pub fn get(&self, name: &str) -> Option<&ServiceInfo> {
        self.services.get(name)
    }

    /// Get mutable service info by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ServiceInfo> {
        self.services.get_mut(name)
    }

    /// Check if it's time for discovery
    pub fn should_discover(&self) -> bool {
        self.last_discovery.elapsed() >= self.discovery_interval
    }

    /// Get all healthy services
    pub fn healthy_services(&self) -> Vec<&ServiceInfo> {
        self.services
            .values()
            .filter(|service| service.is_healthy())
            .collect()
    }

    /// Get all unhealthy services
    pub fn unhealthy_services(&self) -> Vec<&ServiceInfo> {
        self.services
            .values()
            .filter(|service| !service.is_healthy())
            .collect()
    }

    /// Mark discovery time
    pub fn mark_discovery(&mut self) {
        self.last_discovery = Instant::now();
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}
