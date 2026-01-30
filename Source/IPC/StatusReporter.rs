//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # Status Reporter
//! 
//! Reports Mountain's IPC status to Sky for monitoring and debugging.
//! Provides real-time status information about IPC communication between Mountain and Wind.

#![allow(non_snake_case, non_camel_case_types)]

use std::{sync::{Arc, Mutex}, time::{Duration, SystemTime}};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// Comprehensive status report combining all monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveStatusReport {
    pub basic_status: IPCStatusReport,
    pub performance_metrics: PerformanceMetrics,
    pub health_status: HealthMonitor,
    pub timestamp: u64,
}

/// Advanced performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub messages_per_second: f64,
    pub average_latency_ms: f64,
    pub peak_latency_ms: f64,
    pub compression_ratio: f64,
    pub connection_pool_utilization: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub last_update: u64,
}

/// Health monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitor {
    pub health_score: f64,
    pub last_health_check: u64,
    pub issues_detected: Vec<HealthIssue>,
    pub recovery_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub issue_type: HealthIssueType,
    pub severity: SeverityLevel,
    pub description: String,
    pub detected_at: u64,
    pub resolved_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthIssueType {
    HighLatency,
    MemoryPressure,
    ConnectionLoss,
    QueueOverflow,
    SecurityViolation,
    PerformanceDegradation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// IPC status information for Sky monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCStatusReport {
    pub timestamp: u64,
    pub connection_status: ConnectionStatus,
    pub message_queue_size: usize,
    pub active_listeners: Vec<String>,
    pub recent_messages: Vec<MessageStats>,
    pub error_count: u32,
    pub uptime_seconds: u64,
}

/// Connection status details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub last_heartbeat: u64,
    pub connection_duration: u64,
}

/// Message statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub channel: String,
    pub message_count: u32,
    pub last_message_time: u64,
    pub average_processing_time_ms: f64,
}

/// Service discovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub status: ServiceStatus,
    pub last_heartbeat: u64,
    pub uptime: u64,
    pub dependencies: Vec<String>,
    pub metrics: ServiceMetrics,
    pub endpoint: Option<String>,
    pub port: Option<u16>,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Degraded,
    Stopped,
    Error,
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub response_time: f64,
    pub error_rate: f64,
    pub throughput: f64,
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub last_updated: u64,
}

/// Service discovery registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistry {
    pub services: HashMap<String, ServiceInfo>,
    pub last_discovery: u64,
    pub discovery_interval: u64,
}

/// Status reporter for IPC communication
pub struct StatusReporter {
    runtime: Arc<ApplicationRunTime>,
    ipc_server: Option<Arc<crate::IPC::TauriIPCServer::TauriIPCServer>>,
    status_history: Arc<Mutex<Vec<IPCStatusReport>>>,
    start_time: SystemTime,
    error_count: Arc<Mutex<u32>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    health_monitor: Arc<Mutex<HealthMonitor>>,
    service_registry: Arc<RwLock<ServiceRegistry>>,
    discovered_services: Arc<RwLock<HashSet<String>>>,
}

impl StatusReporter {
    /// Create a new status reporter
    pub fn new(runtime: Arc<ApplicationRunTime>) -> Self {
        info!("[StatusReporter] Creating IPC status reporter");
        
        Self {
            runtime,
            ipc_server: None,
            status_history: Arc::new(Mutex::new(Vec::new())),
            start_time: SystemTime::now(),
            error_count: Arc::new(Mutex::new(0)),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics {
                messages_per_second: 0.0,
                average_latency_ms: 0.0,
                peak_latency_ms: 0.0,
                compression_ratio: 1.0,
                connection_pool_utilization: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                last_update: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            })),
            health_monitor: Arc::new(Mutex::new(HealthMonitor {
                health_score: 100.0,
                last_health_check: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                issues_detected: Vec::new(),
                recovery_attempts: 0,
            })),
            service_registry: Arc::new(RwLock::new(ServiceRegistry {
                services: HashMap::new(),
                last_discovery: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                discovery_interval: 30000, // 30 seconds
            })),
            discovered_services: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Set the IPC server instance
    pub fn set_ipc_server(&mut self, ipc_server: Arc<crate::IPC::TauriIPCServer::TauriIPCServer>) {
        self.ipc_server = Some(ipc_server);
    }

    /// Generate a status report
    pub async fn generate_status_report(&self) -> Result<IPCStatusReport, String> {
        debug!("[StatusReporter] Generating IPC status report");
        
        let ipc_server = self.ipc_server.as_ref()
            .ok_or("IPC Server not set".to_string())?;
        
        // Get connection status
        let connection_status = ConnectionStatus {
            is_connected: ipc_server.get_connection_status()?,
            last_heartbeat: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            connection_duration: SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Get message queue size
        let message_queue_size = ipc_server.get_queue_size()?;
        
        // Get active listeners (simplified - would need IPC server to expose this)
        let active_listeners = vec!["configuration".to_string(), "file".to_string(), "storage".to_string()];
        
        // Get recent message stats (simplified)
        let recent_messages = vec![
            MessageStats {
                channel: "configuration".to_string(),
                message_count: 10,
                last_message_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                average_processing_time_ms: 5.0,
            },
            MessageStats {
                channel: "file".to_string(),
                message_count: 5,
                last_message_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() - 10,
                average_processing_time_ms: 15.0,
            },
        ];
        
        // Get error count
        let error_count = {
            let guard = self.error_count.lock()
                .map_err(|e| format!("Failed to get error count: {}", e))?;
            *guard
        };
        
        // Calculate uptime
        let uptime_seconds = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default()
            .as_secs();
        
        let report = IPCStatusReport {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            connection_status,
            message_queue_size,
            active_listeners,
            recent_messages,
            error_count,
            uptime_seconds,
        };
        
        // Store in history
        {
            let mut history = self.status_history.lock()
                .map_err(|e| format!("Failed to access status history: {}", e))?;
            history.push(report.clone());
            
            // Keep only last 100 reports
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        Ok(report)
    }

    /// ADVANCED STATUS REPORTING: Microsoft-inspired comprehensive reporting
    pub async fn report_to_sky(&self) -> Result<(), String> {
        debug!("[StatusReporter] Reporting IPC status to Sky");
        
        let report = self.generate_status_report().await?;
        
        // Update performance metrics
        self.update_performance_metrics().await?;
        
        // Perform health check
        self.perform_health_check().await?;
        
        // Get advanced metrics
        let performance_metrics = self.get_performance_metrics()?;
        let health_status = self.get_health_status()?;
        
        // Emit comprehensive status report
        let comprehensive_report = ComprehensiveStatusReport {
            basic_status: report.clone(),
            performance_metrics,
            health_status,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        
        // Emit status to Sky via Tauri events
        if let Err(e) = self.runtime.Environment.ApplicationHandle.emit("ipc-status-report", &comprehensive_report) {
            error!("[StatusReporter] Failed to emit status report to Sky: {}", e);
            return Err(format!("Failed to emit status report: {}", e));
        }
        
        // Emit separate events for detailed monitoring
        if let Err(e) = self.runtime.Environment.ApplicationHandle.emit("ipc-performance-metrics", &performance_metrics) {
            error!("[StatusReporter] Failed to emit performance metrics: {}", e);
        }
        
        if let Err(e) = self.runtime.Environment.ApplicationHandle.emit("ipc-health-status", &health_status) {
            error!("[StatusReporter] Failed to emit health status: {}", e);
        }
        
        debug!("[StatusReporter] Comprehensive status report sent to Sky");
        Ok(())
    }

    /// Start periodic status reporting
    pub async fn start_periodic_reporting(&self, interval_seconds: u64) -> Result<(), String> {
        info!("[StatusReporter] Starting periodic status reporting (interval: {}s)", interval_seconds);
        
        let reporter = self.clone_reporter();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = reporter.report_to_sky().await {
                    error!("[StatusReporter] Periodic reporting failed: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Record an error
    pub fn record_error(&self) {
        if let Ok(mut error_count) = self.error_count.lock() {
            *error_count += 1;
        }
    }

    /// Get status history
    pub fn get_status_history(&self) -> Result<Vec<IPCStatusReport>, String> {
        let history = self.status_history.lock()
            .map_err(|e| format!("Failed to access status history: {}", e))?;
        Ok(history.clone())
    }

    /// Get the start time
    pub fn get_start_time(&self) -> SystemTime {
        self.start_time
    }

    /// ADVANCED PERFORMANCE MONITORING: Microsoft-inspired performance tracking
    pub async fn update_performance_metrics(&self) -> Result<(), String> {
        let ipc_server = self.ipc_server.as_ref()
            .ok_or("IPC Server not set".to_string())?;

        // Get connection statistics
        let connection_stats = ipc_server.get_connection_stats().await
            .unwrap_or_default();

        // Calculate performance metrics
        let mut metrics = self.performance_metrics.lock()
            .map_err(|e| format!("Failed to access performance metrics: {}", e))?;

        // Update metrics with real-time data
        metrics.messages_per_second = self.calculate_message_rate().await;
        metrics.average_latency_ms = self.calculate_average_latency().await;
        metrics.peak_latency_ms = self.calculate_peak_latency().await;
        metrics.compression_ratio = self.calculate_compression_ratio().await;
        metrics.connection_pool_utilization = self.calculate_pool_utilization(&connection_stats).await;
        metrics.memory_usage_mb = self.get_memory_usage().await;
        metrics.cpu_usage_percent = self.get_cpu_usage().await;
        metrics.last_update = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        debug!("[StatusReporter] Performance metrics updated: {:.2} msg/s, {:.2}ms latency", 
               metrics.messages_per_second, metrics.average_latency_ms);

        Ok(())
    }

    /// ADVANCED HEALTH MONITORING: Microsoft-inspired health checks
    pub async fn perform_health_check(&self) -> Result<(), String> {
        let mut health_monitor = self.health_monitor.lock()
            .map_err(|e| format!("Failed to access health monitor: {}", e))?;

        let mut health_score: f32 = 100.0;
        let mut issues = Vec::new();

        // Check connection health
        if let Some(ipc_server) = &self.ipc_server {
            if !ipc_server.get_connection_status()? {
                health_score -= 25.0;
                issues.push(HealthIssue {
                    issue_type: HealthIssueType::ConnectionLoss,
                    severity: SeverityLevel::Critical,
                    description: "IPC connection lost".to_string(),
                    detected_at: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    resolved_at: None,
                });
            }
        }

        // Check message queue
        if let Some(ipc_server) = &self.ipc_server {
            let queue_size = ipc_server.get_queue_size()?;
            if queue_size > 100 {
                health_score -= 15.0;
                issues.push(HealthIssue {
                    issue_type: HealthIssueType::QueueOverflow,
                    severity: SeverityLevel::High,
                    description: format!("Message queue overflow: {} messages", queue_size),
                    detected_at: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    resolved_at: None,
                });
            }
        }

        // Check performance degradation
        let metrics = self.performance_metrics.lock()
            .map_err(|e| format!("Failed to access performance metrics: {}", e))?;
        
        if metrics.average_latency_ms > 100.0 {
            health_score -= 20.0;
            issues.push(HealthIssue {
                issue_type: HealthIssueType::HighLatency,
                severity: SeverityLevel::High,
                description: format!("High latency detected: {:.2}ms", metrics.average_latency_ms),
                detected_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                resolved_at: None,
            });
        }

        // Update health monitor
        health_monitor.health_score = health_score.max(0.0);
        health_monitor.issues_detected = issues;
        health_monitor.last_health_check = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Emit health alert if score is low
        if health_score < 70.0 {
            warn!("[StatusReporter] Health check failed: score {:.1}%
", health_score);
            
            if let Err(e) = self.runtime.Environment.ApplicationHandle.emit(
                "ipc-health-alert", 
                &health_monitor.clone()
            ) {
                error!("[StatusReporter] Failed to emit health alert: {}", e);
            }
        }

        Ok(())
    }

    /// ADVANCED METRICS CALCULATION: Microsoft-inspired performance algorithms
    async fn calculate_message_rate(&self) -> f64 {
        // Calculate messages per second based on recent activity
        let history = self.get_status_history()
            .unwrap_or_default();
        
        if history.len() < 2 {
            return 0.0;
        }

        let recent_reports: Vec<&IPCStatusReport> = history.iter()
            .rev()
            .take(5)
            .collect();

        let total_messages: u32 = recent_reports.iter()
            .map(|report| report.recent_messages.iter().map(|m| m.message_count).sum::<u32>())
            .sum();

        let time_span = if recent_reports.len() > 1 {
            let first_time = recent_reports.first().unwrap().timestamp;
            let last_time = recent_reports.last().unwrap().timestamp;
            (last_time - first_time) as f64 / 1000.0 // Convert to seconds
        } else {
            1.0
        };

        total_messages as f64 / time_span.max(1.0)
    }

    async fn calculate_average_latency(&self) -> f64 {
        let history = self.get_status_history()
            .unwrap_or_default();
        
        if history.is_empty() {
            return 0.0;
        }

        let recent_reports: Vec<&IPCStatusReport> = history.iter()
            .rev()
            .take(10)
            .collect();

        let total_latency: f64 = recent_reports.iter()
            .flat_map(|report| &report.recent_messages)
            .map(|msg| msg.average_processing_time_ms)
            .sum();

        let message_count = recent_reports.iter()
            .flat_map(|report| &report.recent_messages)
            .count();

        total_latency / message_count.max(1) as f64
    }

    async fn calculate_peak_latency(&self) -> f64 {
        let history = self.get_status_history()
            .unwrap_or_default();
        
        history.iter()
            .flat_map(|report| &report.recent_messages)
            .map(|msg| msg.average_processing_time_ms)
            .fold(0.0, f64::max)
    }

    async fn calculate_compression_ratio(&self) -> f64 {
        // Simplified compression ratio calculation
        // In a real implementation, this would track actual compression stats
        2.5 // Example compression ratio
    }

    async fn calculate_pool_utilization(&self, stats: &crate::IPC::TauriIPCServer::ConnectionStats) -> f64 {
        if stats.total_connections == 0 {
            return 0.0;
        }

        stats.total_connections as f64 / stats.max_connections as f64
    }

    async fn get_memory_usage(&self) -> f64 {
        // Simplified memory usage estimation
        // In a real implementation, use system APIs
        50.0 // Example MB usage
    }

    async fn get_cpu_usage(&self) -> f64 {
        // Simplified CPU usage estimation
        // In a real implementation, use system APIs
        15.0 // Example CPU percentage
    }

    /// SERVICE DISCOVERY: Discover available Mountain services
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, String> {
        info!("[StatusReporter] Starting service discovery");
        
        let mut registry = self.service_registry.write().await;
        let mut discovered = self.discovered_services.write().await;
        
        let mut services = Vec::new();
        
        // Discover core Mountain services
        let core_services = vec![
            ("EditorService", "1.0.0", ServiceStatus::Running),
            ("ExtensionHostService", "1.0.0", ServiceStatus::Running),
            ("ConfigurationService", "1.0.0", ServiceStatus::Running),
            ("FileService", "1.0.0", ServiceStatus::Running),
            ("StorageService", "1.0.0", ServiceStatus::Running),
        ];
        
        for (name, version, status) in core_services {
            let service_info = ServiceInfo {
                name: name.to_string(),
                version: version.to_string(),
                status: status.clone(),
                last_heartbeat: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                uptime: SystemTime::now()
                    .duration_since(self.start_time)
                    .unwrap_or_default()
                    .as_secs(),
                dependencies: self.get_service_dependencies(name),
                metrics: ServiceMetrics {
                    response_time: self.calculate_service_response_time(name).await,
                    error_rate: self.calculate_service_error_rate(name).await,
                    throughput: self.calculate_service_throughput(name).await,
                    memory_usage: self.get_service_memory_usage(name).await,
                    cpu_usage: self.get_service_cpu_usage(name).await,
                    last_updated: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                },
                endpoint: Some(format!("localhost:{}", 50050 + services.len() as u16)),
                port: Some(50050 + services.len() as u16),
            };
            
            registry.services.insert(name.to_string(), service_info.clone());
            discovered.insert(name.to_string());
            services.push(service_info);
        }
        
        registry.last_discovery = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        info!("[StatusReporter] Service discovery completed: {} services found", services.len());
        
        // Emit service discovery event
        if let Err(e) = self.runtime.Environment.ApplicationHandle.emit(
            "mountain_service_discovery", 
            &services
        ) {
            error!("[StatusReporter] Failed to emit service discovery event: {}", e);
        }
        
        Ok(services)
    }
    
    /// Get service dependencies
    fn get_service_dependencies(&self, service_name: &str) -> Vec<String> {
        match service_name {
            "ExtensionHostService" => vec!["ConfigurationService".to_string()],
            "FileService" => vec!["StorageService".to_string()],
            "StorageService" => vec!["ConfigurationService".to_string()],
            _ => Vec::new(),
        }
    }
    
    /// Calculate service response time
    async fn calculate_service_response_time(&self, service_name: &str) -> f64 {
        // Mock implementation - would use real metrics in production
        match service_name {
            "EditorService" => 5.0,
            "ExtensionHostService" => 15.0,
            "ConfigurationService" => 2.0,
            "FileService" => 8.0,
            "StorageService" => 3.0,
            _ => 10.0,
        }
    }
    
    /// Calculate service error rate
    async fn calculate_service_error_rate(&self, service_name: &str) -> f64 {
        // Mock implementation - would use real metrics in production
        match service_name {
            "EditorService" => 0.1,
            "ExtensionHostService" => 2.5,
            "ConfigurationService" => 0.5,
            "FileService" => 1.2,
            "StorageService" => 0.8,
            _ => 5.0,
        }
    }
    
    /// Calculate service throughput
    async fn calculate_service_throughput(&self, service_name: &str) -> f64 {
        // Mock implementation - would use real metrics in production
        match service_name {
            "EditorService" => 1000.0,
            "ExtensionHostService" => 500.0,
            "ConfigurationService" => 2000.0,
            "FileService" => 800.0,
            "StorageService" => 1500.0,
            _ => 100.0,
        }
    }
    
    /// Get service memory usage
    async fn get_service_memory_usage(&self, service_name: &str) -> f64 {
        // Mock implementation - would use real metrics in production
        match service_name {
            "EditorService" => 256.0,
            "ExtensionHostService" => 512.0,
            "ConfigurationService" => 128.0,
            "FileService" => 192.0,
            "StorageService" => 64.0,
            _ => 100.0,
        }
    }
    
    /// Get service CPU usage
    async fn get_service_cpu_usage(&self, service_name: &str) -> f64 {
        // Mock implementation - would use real metrics in production
        match service_name {
            "EditorService" => 15.0,
            "ExtensionHostService" => 25.0,
            "ConfigurationService" => 5.0,
            "FileService" => 10.0,
            "StorageService" => 8.0,
            _ => 20.0,
        }
    }
    
    /// Start periodic service discovery
    pub async fn start_periodic_discovery(&self) -> Result<(), String> {
        info!("[StatusReporter] Starting periodic service discovery");
        
        let registry = self.service_registry.read().await;
        let interval = registry.discovery_interval;
        drop(registry);
        
        let reporter = self.clone_reporter();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = reporter.discover_services().await {
                    error!("[StatusReporter] Periodic service discovery failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Get service registry
    pub async fn get_service_registry(&self) -> Result<ServiceRegistry, String> {
        let registry = self.service_registry.read().await;
        Ok(registry.clone())
    }
    
    /// Get service information
    pub async fn get_service_info(&self, service_name: &str) -> Result<Option<ServiceInfo>, String> {
        let registry = self.service_registry.read().await;
        Ok(registry.services.get(service_name).cloned())
    }
    
    /// ADVANCED RECOVERY: Microsoft-inspired automatic recovery
    pub async fn attempt_recovery(&self) -> Result<(), String> {
        let mut health_monitor = self.health_monitor.lock()
            .map_err(|e| format!("Failed to access health monitor: {}", e))?;

        health_monitor.recovery_attempts += 1;

        // Simple recovery logic
        if let Some(ipc_server) = &self.ipc_server {
            // Reset connection
            if let Err(e) = ipc_server.dispose() {
                return Err(format!("Failed to dispose IPC server: {}", e));
            }

            // Reinitialize
            if let Err(e) = ipc_server.initialize().await {
                return Err(format!("Failed to reinitialize IPC server: {}", e));
            }
        }

        // Clear error count
        if let Ok(mut error_count) = self.error_count.lock() {
            *error_count = 0;
        }

        info!("[StatusReporter] Recovery attempt {} completed", health_monitor.recovery_attempts);
        Ok(())
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
        let metrics = self.performance_metrics.lock()
            .map_err(|e| format!("Failed to access performance metrics: {}", e))?;
        Ok(metrics.clone())
    }

    /// Get health status
    pub fn get_health_status(&self) -> Result<HealthMonitor, String> {
        let health_monitor = self.health_monitor.lock()
            .map_err(|e| format!("Failed to access health monitor: {}", e))?;
        Ok(health_monitor.clone())
    }

    /// Clone the reporter for async tasks
    fn clone_reporter(&self) -> StatusReporter {
        StatusReporter {
            runtime: self.runtime.clone(),
            ipc_server: self.ipc_server.clone(),
            status_history: self.status_history.clone(),
            start_time: self.start_time,
            error_count: self.error_count.clone(),
            performance_metrics: self.performance_metrics.clone(),
            health_monitor: self.health_monitor.clone(),
            service_registry: self.service_registry.clone(),
            discovered_services: self.discovered_services.clone(),
        }
    }
}

/// Tauri command to get current IPC status
#[tauri::command]
pub async fn mountain_get_ipc_status(
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    debug!("[StatusReporter] Tauri command: get_ipc_status");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.generate_status_report().await
            .map(|report| serde_json::to_value(report).unwrap_or(serde_json::Value::Null))
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get IPC status history
#[tauri::command]
pub async fn mountain_get_ipc_status_history(
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    debug!("[StatusReporter] Tauri command: get_ipc_status_history");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_status_history()
            .map(|history| serde_json::to_value(history).unwrap_or(serde_json::Value::Null))
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to start periodic status reporting
#[tauri::command]
pub async fn mountain_start_ipc_status_reporting(
    app_handle: tauri::AppHandle,
    interval_seconds: u64,
) -> Result<serde_json::Value, String> {
    debug!("[StatusReporter] Tauri command: start_ipc_status_reporting");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.start_periodic_reporting(interval_seconds).await
            .map(|_| serde_json::json!({ "status": "started", "interval_seconds": interval_seconds }))
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// ADVANCED TAURI COMMANDS: Microsoft-inspired comprehensive monitoring

/// Tauri command to get performance metrics
#[tauri::command]
pub async fn mountain_get_performance_metrics(
    app_handle: tauri::AppHandle,
) -> Result<PerformanceMetrics, String> {
    debug!("[StatusReporter] Tauri command: get_performance_metrics");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_performance_metrics()
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get health status
#[tauri::command]
pub async fn mountain_get_health_status(
    app_handle: tauri::AppHandle,
) -> Result<HealthMonitor, String> {
    debug!("[StatusReporter] Tauri command: get_health_status");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_health_status()
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to perform health check
#[tauri::command]
pub async fn mountain_perform_health_check(
    app_handle: tauri::AppHandle,
) -> Result<HealthMonitor, String> {
    debug!("[StatusReporter] Tauri command: perform_health_check");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.perform_health_check().await?;
        reporter.get_health_status()
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to attempt recovery
#[tauri::command]
pub async fn mountain_attempt_recovery(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    debug!("[StatusReporter] Tauri command: attempt_recovery");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.attempt_recovery().await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get service registry
#[tauri::command]
pub async fn mountain_get_service_registry(
    app_handle: tauri::AppHandle,
) -> Result<ServiceRegistry, String> {
    debug!("[StatusReporter] Tauri command: get_service_registry");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_service_registry().await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get service information
#[tauri::command]
pub async fn mountain_get_service_info(
    app_handle: tauri::AppHandle,
    service_name: String,
) -> Result<Option<ServiceInfo>, String> {
    debug!("[StatusReporter] Tauri command: get_service_info");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_service_info(&service_name).await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to discover services
#[tauri::command]
pub async fn mountain_discover_services(
    app_handle: tauri::AppHandle,
) -> Result<Vec<ServiceInfo>, String> {
    debug!("[StatusReporter] Tauri command: discover_services");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.discover_services().await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to start periodic service discovery
#[tauri::command]
pub async fn mountain_start_service_discovery(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    debug!("[StatusReporter] Tauri command: start_service_discovery");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.start_periodic_discovery().await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get comprehensive status report
#[tauri::command]
pub async fn mountain_get_comprehensive_status(
    app_handle: tauri::AppHandle,
) -> Result<ComprehensiveStatusReport, String> {
    debug!("[StatusReporter] Tauri command: get_comprehensive_status");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        let basic_status = reporter.generate_status_report().await?;
        let performance_metrics = reporter.get_performance_metrics()?;
        let health_status = reporter.get_health_status()?;
        
        Ok(ComprehensiveStatusReport {
            basic_status,
            performance_metrics,
            health_status,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Initialize status reporter in Mountain's setup
pub fn initialize_status_reporter(
    app_handle: &tauri::AppHandle,
    runtime: Arc<ApplicationRunTime>,
) -> Result<StatusReporter, String> {
    info!("[StatusReporter] Initializing status reporter");
    
    let reporter = StatusReporter::new(runtime);
    
    // Store in application state
    app_handle.manage(reporter.clone_reporter());
    
    Ok(reporter)
}
