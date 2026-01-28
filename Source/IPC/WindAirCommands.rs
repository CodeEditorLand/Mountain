//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # Wind-Air Commands
//!
//! Tauri IPC commands that allow Wind (frontend) to delegate background operations to Air (daemon).
//! All commands delegate to AirClient via gRPC and return appropriate DTOs or friendly error messages.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, command, Manager};
use tonic::transport::Channel;
use log::{debug, error, info};

use crate::Configuration::MountainConfiguration::MountainConfiguration;

// Re-import Air's generated gRPC client
// Note: In actual implementation, this would be imported from Air's crate or a shared dependency
// For now, we'll define the client interface inline or use a wrapper
use air_proto::air_client::AirClient as AirServiceClient;
use air_proto::{AuthenticationRequest, UpdateCheckRequest, DownloadRequest, 
                IndexRequest, SearchRequest, StatusRequest, MetricsRequest};

/// Data Transfer Objects for Wind-Air communication

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfoDTO {
    pub update_available: bool,
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResultDTO {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponseDTO {
    pub success: bool,
    pub token: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResultDTO {
    pub success: bool,
    pub files_indexed: u32,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsDTO {
    pub results: Vec<FileResultDTO>,
    pub total_results: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResultDTO {
    pub path: String,
    pub size: u64,
    pub line: Option<u32>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirServiceStatusDTO {
    pub version: String,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_requests: u32,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirMetricsDTO {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub average_response_time: f64,
    pub disk_usage_mb: f64,
    pub network_usage_mbps: f64,
}

/// Air Client - Wrapper for the gRPC client connection to Air daemon
#[derive(Debug, Clone)]
pub struct AirClient {
    client: AirServiceClient<Channel>,
}

impl AirClient {
    /// Create a new AirClient connected to the Air daemon
    pub async fn new(address: String) -> Result<Self, String> {
        debug!("[WindAirCommands] Connecting to Air daemon at: {}", address);
        
        let client = AirServiceClient::connect(address)
            .await
            .map_err(|e| format!("Failed to connect to Air daemon: {}", e))?;
        
        info!("[WindAirCommands] Successfully connected to Air daemon");
        Ok(Self { client })
    }
    
    /// Reconnect to Air daemon
    pub async fn reconnect(&mut self, address: String) -> Result<(), String> {
        debug!("[WindAirCommands] Reconnecting to Air daemon at: {}", address);
        
        self.client = AirServiceClient::connect(address)
            .await
            .map_err(|e| format!("Failed to reconnect to Air daemon: {}", e))?;
        
        info!("[WindAirCommands] Successfully reconnected to Air daemon");
        Ok(())
    }
}

// ============================================================================
// Tauri IPC Commands for Wind-Air Communication
// ============================================================================

/// Command: Check for Updates
/// 
/// Checks if a newer version of the application is available.
/// Delegates to Air's update checking service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `current_version` - Current application version
/// * `channel` - Update channel ("stable", "beta", "nightly")
///
/// # Returns
/// `UpdateInfoDTO` with update information or error message
#[tauri::command]
pub async fn CheckForUpdates(
    app_handle: AppHandle,
    current_version: Option<String>,
    channel: Option<String>,
) -> Result<UpdateInfoDTO, String> {
    debug!("[WindAirCommands] CheckForUpdates called with version: {:?}, channel: {:?}", 
           current_version, channel);
    
    // Get the Air client from app state or configuration
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    // Build the request
    let request = UpdateCheckRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        current_version: current_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
        channel: channel.unwrap_or_else(|| "stable".to_string()),
    };
    
    // Delegate to Air via gRPC
    let response = client.client
        .check_for_updates(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Update check failed: {}", e))?
        .into_inner();
    
    // Check for errors in the response
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    let result = UpdateInfoDTO {
        update_available: response.update_available,
        version: response.version,
        download_url: response.download_url,
        release_notes: response.release_notes,
    };
    
    info!("[WindAirCommands] Update check completed: available={}", result.update_available);
    Ok(result)
}

/// Command: Download Update
///
/// Downloads an application update from the specified URL.
/// Delegates to Air's download service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `url` - URL to download the update from
/// * `destination` - Local destination path for the download
/// * `checksum` - Optional SHA256 checksum for verification
///
/// # Returns
/// `DownloadResultDTO` with download status
#[tauri::command]
pub async fn DownloadUpdate(
    app_handle: AppHandle,
    url: String,
    destination: String,
    checksum: Option<String>,
) -> Result<DownloadResultDTO, String> {
    debug!("[WindAirCommands] DownloadUpdate called: {} -> {}", url, destination);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = DownloadRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        url,
        destination_path: destination,
        checksum: checksum.unwrap_or_default(),
        headers: Default::default(),
    };
    
    let response = client.client
        .download_file(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Update download failed: {}", e))?
        .into_inner();
    
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    let result = DownloadResultDTO {
        success: response.success,
        file_path: response.file_path,
        file_size: response.file_size,
        checksum: response.checksum,
    };
    
    info!("[WindAirCommands] Update download completed: success={}", result.success);
    Ok(result)
}

/// Command: Apply Update
///
/// Applies a downloaded update to the application.
/// Delegates to Air's update installation service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `update_id` - Identifier of the update to apply
/// * `update_path` - Path to the update package
///
/// # Returns
/// Success status or error message
#[tauri::command]
pub async fn ApplyUpdate(
    app_handle: AppHandle,
    update_id: String,
    update_path: String,
) -> Result<bool, String> {
    debug!("[WindAirCommands] ApplyUpdate called: id={}, path={}", update_id, update_path);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    // Use ApplyUpdateRequest from Air's proto
    let request = air_proto::ApplyUpdateRequest {
        request_id: update_id.clone(),
        version: update_id,
        update_path,
    };
    
    let response = client.client
        .apply_update(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Update application failed: {}", e))?
        .into_inner();
    
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    info!("[WindAirCommands] Update applied successfully");
    Ok(response.success)
}

/// Command: Download File
///
/// Downloads any file from a URL.
/// Delegates to Air's download service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `url` - URL to download from
/// * `destination` - Local destination path
///
/// # Returns
/// `DownloadResultDTO` with download status
#[tauri::command]
pub async fn DownloadFile(
    app_handle: AppHandle,
    url: String,
    destination: String,
) -> Result<DownloadResultDTO, String> {
    debug!("[WindAirCommands] DownloadFile called: {} -> {}", url, destination);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = DownloadRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        url,
        destination_path: destination,
        checksum: String::new(),
        headers: Default::default(),
    };
    
    let response = client.client
        .download_file(tonic::Request::new(request))
        .await
        .map_err(|e| format!("File download failed: {}", e))?
        .into_inner();
    
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    let result = DownloadResultDTO {
        success: response.success,
        file_path: response.file_path,
        file_size: response.file_size,
        checksum: response.checksum,
    };
    
    info!("[WindAirCommands] File download completed");
    Ok(result)
}

/// Command: Authenticate User
///
/// Authenticates a user with the specified provider.
/// Delegates to Air's authentication service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `username` - User's username/email
/// * `password` - User's password (or auth token)
/// * `provider` - Auth provider ("github", "gitlab", "microsoft", etc.)
///
/// # Returns
/// `AuthResponseDTO` with authentication token
#[tauri::command]
pub async fn AuthenticateUser(
    app_handle: AppHandle,
    username: String,
    password: String,
    provider: String,
) -> Result<AuthResponseDTO, String> {
    debug!("[WindAirCommands] AuthenticateUser called: {} via {}", username, provider);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = AuthenticationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        username,
        password,
        provider,
    };
    
    let response = client.client
        .authenticate(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Authentication failed: {}", e))?
        .into_inner();
    
    if !response.success && !response.error.is_empty() {
        return Err(response.error);
    }
    
    let result = AuthResponseDTO {
        success: response.success,
        token: response.token,
        error: if response.error.is_empty() { None } else { Some(response.error) },
    };
    
    info!("[WindAirCommands] Authentication completed: success={}", result.success);
    Ok(result)
}

/// Command: Index Files
///
/// Initiates file indexing for a directory.
/// Delegates to Air's file indexing service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `path` - Path to directory to index
/// * `patterns` - File patterns to include
/// * `exclude_patterns` - File patterns to exclude
/// * `max_depth` - Maximum directory depth to traverse
///
/// # Returns
/// `IndexResultDTO` with indexing results
#[tauri::command]
pub async fn IndexFiles(
    app_handle: AppHandle,
    path: String,
    patterns: Vec<String>,
    exclude_patterns: Option<Vec<String>>,
    max_depth: Option<u32>,
) -> Result<IndexResultDTO, String> {
    debug!("[WindAirCommands] IndexFiles called: {} with patterns: {:?}", path, patterns);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = IndexRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        path,
        patterns,
        exclude_patterns: exclude_patterns.unwrap_or_default(),
        max_depth: max_depth.unwrap_or(100),
    };
    
    let response = client.client
        .index_files(tonic::Request::new(request))
        .await
        .map_err(|e| format!("File indexing failed: {}", e))?
        .into_inner();
    
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    let result = IndexResultDTO {
        success: response.success,
        files_indexed: response.files_indexed,
        total_size: response.total_size,
    };
    
    info!("[WindAirCommands] File indexing completed: {} files", result.files_indexed);
    Ok(result)
}

/// Command: Search Files
///
/// Searches previously indexed files.
/// Delegates to Air's search service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `query` - Search query string
/// * `index_id` - Index identifier (or path for backward compatibility)
/// * `max_results` - Maximum number of results to return
///
/// # Returns
/// `SearchResultsDTO` with matching files
#[tauri::command]
pub async fn SearchFiles(
    app_handle: AppHandle,
    query: String,
    index_id: Option<String>,
    max_results: Option<u32>,
) -> Result<SearchResultsDTO, String> {
    debug!("[WindAirCommands] SearchFiles called: query={}, index={:?}", query, index_id);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = SearchRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        query,
        path: index_id.unwrap_or_else(|| String::from(".")),
        max_results: max_results.unwrap_or(100),
    };
    
    let response = client.client
        .search_files(tonic::Request::new(request))
        .await
        .map_err(|e| format!("File search failed: {}", e))?
        .into_inner();
    
    if !response.error.is_empty() {
        return Err(response.error);
    }
    
    let results: Vec<FileResultDTO> = response.results
        .into_iter()
        .map(|r| FileResultDTO {
            path: r.path,
            size: r.size,
            line: if r.line > 0 { Some(r.line) } else { None },
            content: if !r.content.is_empty() { Some(r.content) } else { None },
        })
        .collect();
    
    let result = SearchResultsDTO {
        results,
        total_results: response.total_results,
    };
    
    info!("[WindAirCommands] File search completed: {} results", result.total_results);
    Ok(result)
}

/// Command: Get Air Status
///
/// Retrieves the current status of the Air daemon.
/// Delegates to Air's status service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
///
/// # Returns
/// `AirServiceStatusDTO` with service status information
#[tauri::command]
pub async fn GetAirStatus(
    app_handle: AppHandle,
) -> Result<AirServiceStatusDTO, String> {
    debug!("[WindAirCommands] GetAirStatus called");
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = StatusRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
    };
    
    let response = client.client
        .get_status(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Failed to get Air status: {}", e))?
        .into_inner();
    
    // Also perform health check
    let health_response = client.client
        .health_check(tonic::Request::new(air_proto::HealthCheckRequest {}))
        .await
        .map_err(|e| format!("Health check failed: {}", e))?
        .into_inner();
    
    let result = AirServiceStatusDTO {
        version: response.version,
        uptime_seconds: response.uptime_seconds,
        total_requests: response.total_requests,
        successful_requests: response.successful_requests,
        failed_requests: response.failed_requests,
        active_requests: response.active_requests,
        healthy: health_response.healthy,
    };
    
    debug!("[WindAirCommands] Air status retrieved: version={}, healthy={}", 
           result.version, result.healthy);
    Ok(result)
}

/// Command: Get Air Metrics
///
/// Retrieves performance and resource metrics from Air.
/// Delegates to Air's metrics service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `metric_type` - Type of metrics ("all", "performance", "resources", "requests")
///
/// # Returns
/// `AirMetricsDTO` with metrics data
#[tauri::command]
pub async fn GetAirMetrics(
    app_handle: AppHandle,
    metric_type: Option<String>,
) -> Result<AirMetricsDTO, String> {
    debug!("[WindAirCommands] GetAirMetrics called with type: {:?}", metric_type);
    
    let air_address = get_air_address(&app_handle)?;
    let mut client = get_or_create_air_client(&app_handle, air_address).await?;
    
    let request = MetricsRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        metric_type: metric_type.unwrap_or_else(|| "all".to_string()),
    };
    
    let response = client.client
        .get_metrics(tonic::Request::new(request))
        .await
        .map_err(|e| format!("Failed to get Air metrics: {}", e))?
        .into_inner();
    
    let result = AirMetricsDTO {
        memory_usage_mb: response.memory_usage_mb,
        cpu_usage_percent: response.cpu_usage_percent,
        average_response_time: response.average_response_time,
        disk_usage_mb: response.disk_usage_mb,
        network_usage_mbps: response.network_usage_mbps,
    };
    
    debug!("[WindAirCommands] Air metrics retrieved");
    Ok(result)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the Air daemon address from configuration
fn get_air_address(app_handle: &AppHandle) -> Result<String, String> {
    // Try to get from Mountain configuration
    if let Some(config) = app_handle.try_state::<Arc<MountainConfiguration>>() {
        let config = config.read().map_err(|e| format!("Failed to read config: {}", e))?;
        // Assume there's an air_connection field or similar
        // For now, return default
        return Ok("[::1]:50053".to_string());
    }
    
    Default configuration
    Ok("[::1]:50053".to_string())
}

/// Get or create the Air client instance
async fn get_or_create_air_client(
    app_handle: &AppHandle,
    address: String,
) -> Result<Arc<tokio::sync::Mutex<AirClient>>, String> {
    // Try to get existing client from app state
    // If not exists, create new one and store it
    
    // For simplicity, create a new client each time
    // In production, you'd use a state management pattern
    let client = AirClient::new(address).await?;
    Ok(Arc::new(tokio::sync::Mutex::new(client)))
}

/// Register all Wind-Air commands with Tauri
pub fn register_wind_air_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .invoke_handler(tauri::generate_handler![
            CheckForUpdates,
            DownloadUpdate,
            ApplyUpdate,
            DownloadFile,
            AuthenticateUser,
            IndexFiles,
            SearchFiles,
            GetAirStatus,
            GetAirMetrics,
        ])
}

// ============================================================================
// Placeholder Module for Air Proto (Would be imported from Air crate)
// ============================================================================

// This is a placeholder - in the actual implementation, these would be imported
// from the Air crate's generated protobuf code
pub mod air_proto {
    use tonic::transport::Channel;
    
    pub mod air_client {
        use super::*;
        
        #[derive(Debug, Clone)]
        pub struct AirClient<T> {
            pub client: super::AirServiceClient<T>,
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct AirServiceClient<T> {
        _phantom: std::marker::PhantomData<T>,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct AuthenticationRequest {
        pub request_id: String,
        pub username: String,
        pub password: String,
        pub provider: String,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct UpdateCheckRequest {
        pub request_id: String,
        pub current_version: String,
        pub channel: String,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct DownloadRequest {
        pub request_id: String,
        pub url: String,
        pub destination_path: String,
        pub checksum: String,
        pub headers: std::collections::HashMap<String, String>,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct ApplyUpdateRequest {
        pub request_id: String,
        pub version: String,
        pub update_path: String,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct IndexRequest {
        pub request_id: String,
        pub path: String,
        pub patterns: Vec<String>,
        pub exclude_patterns: Vec<String>,
        pub max_depth: u32,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct SearchRequest {
        pub request_id: String,
        pub query: String,
        pub path: String,
        pub max_results: u32,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct StatusRequest {
        pub request_id: String,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct MetricsRequest {
        pub request_id: String,
        pub metric_type: String,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct HealthCheckRequest {
        _private: (),
    }
    
    // Response types
    #[derive(Debug, Clone)]
    pub struct UpdateCheckResponse {
        pub request_id: String,
        pub update_available: bool,
        pub version: String,
        pub download_url: String,
        pub release_notes: String,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct ApplyUpdateResponse {
        pub request_id: String,
        pub success: bool,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct DownloadResponse {
        pub request_id: String,
        pub success: bool,
        pub file_path: String,
        pub file_size: u64,
        pub checksum: String,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct AuthenticationResponse {
        pub request_id: String,
        pub success: bool,
        pub token: String,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct IndexResponse {
        pub request_id: String,
        pub success: bool,
        pub files_indexed: u32,
        pub total_size: u64,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct SearchResponse {
        pub request_id: String,
        pub results: Vec<FileResult>,
        pub total_results: u32,
        pub error: String,
    }
    
    #[derive(Debug, Clone)]
    pub struct StatusResponse {
        pub version: String,
        pub uptime_seconds: u64,
        pub total_requests: u64,
        pub successful_requests: u64,
        pub failed_requests: u64,
        pub average_response_time: f64,
        pub memory_usage_mb: f64,
        pub cpu_usage_percent: f64,
        pub active_requests: u32,
    }
    
    #[derive(Debug, Clone)]
    pub struct MetricsResponse {
        pub memory_usage_mb: f64,
        pub cpu_usage_percent: f64,
        pub average_response_time: f64,
        pub disk_usage_mb: f64,
        pub network_usage_mbps: f64,
    }
    
    #[derive(Debug, Clone)]
    pub struct HealthCheckResponse {
        pub healthy: bool,
        pub timestamp: u64,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct FileResult {
        pub path: String,
        pub size: u64,
        pub line: u32,
        pub content: String,
    }
}
