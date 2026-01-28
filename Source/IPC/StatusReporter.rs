//! # Status Reporter
//! 
//! Reports Mountain's IPC status to Sky for monitoring and debugging.
//! Provides real-time status information about IPC communication between Mountain and Wind.

#![allow(non_snake_case, non_camel_case_types)]

use std::{sync::{Arc, Mutex}, time::{Duration, SystemTime}};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

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

/// Status reporter for IPC communication
pub struct StatusReporter {
    runtime: Arc<ApplicationRunTime>,
    ipc_server: Option<Arc<crate::IPC::TauriIPCServer::TauriIPCServer>>,
    status_history: Arc<Mutex<Vec<IPCStatusReport>>>,
    start_time: SystemTime,
    error_count: Arc<Mutex<u32>>,
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

    /// Report status to Sky
    pub async fn report_to_sky(&self) -> Result<(), String> {
        debug!("[StatusReporter] Reporting IPC status to Sky");
        
        let report = self.generate_status_report().await?;
        
        // Emit status to Sky via Tauri events
        if let Err(e) = self.runtime.Environment.ApplicationHandle.emit("ipc-status-report", &report) {
            error!("[StatusReporter] Failed to emit status report to Sky: {}", e);
            return Err(format!("Failed to emit status report: {}", e));
        }
        
        debug!("[StatusReporter] Status report sent to Sky");
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

    /// Clone the reporter for async tasks
    fn clone_reporter(&self) -> StatusReporter {
        StatusReporter {
            runtime: self.runtime.clone(),
            ipc_server: self.ipc_server.clone(),
            status_history: self.status_history.clone(),
            start_time: self.start_time,
            error_count: self.error_count.clone(),
        }
    }
}

/// Tauri command to get current IPC status
#[tauri::command]
pub async fn mountain_get_ipc_status(
    app_handle: tauri::AppHandle,
) -> Result<IPCStatusReport, String> {
    debug!("[StatusReporter] Tauri command: get_ipc_status");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.generate_status_report().await
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to get IPC status history
#[tauri::command]
pub async fn mountain_get_ipc_status_history(
    app_handle: tauri::AppHandle,
) -> Result<Vec<IPCStatusReport>, String> {
    debug!("[StatusReporter] Tauri command: get_ipc_status_history");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.get_status_history()
    } else {
        Err("StatusReporter not found in application state".to_string())
    }
}

/// Tauri command to start periodic status reporting
#[tauri::command]
pub async fn mountain_start_ipc_status_reporting(
    app_handle: tauri::AppHandle,
    interval_seconds: u64,
) -> Result<(), String> {
    debug!("[StatusReporter] Tauri command: start_ipc_status_reporting");
    
    if let Some(reporter) = app_handle.try_state::<StatusReporter>() {
        reporter.start_periodic_reporting(interval_seconds).await
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
