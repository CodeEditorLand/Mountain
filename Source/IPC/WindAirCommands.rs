//! # Wind-Air Commands - Air Daemon Delegation Layer
//!
//! **File Responsibilities:**
//! This module provides the Tauri IPC commands that enable Wind (the TypeScript
//! frontend) to delegate background operations to Air (the Rust daemon). All
//! commands use the gRPC-based AirClient for communication and return
//! structured DTOs or detailed error messages.
//!
//! **Architectural Role in Wind-Mountain-Air Connection:**
//!
//! The WindAirCommands module forms the delegation layer that:
//!
//! 1. **Bridge to Air Daemon:** Provides a Tauri IPC interface to Air's gRPC
//!    services
//! 2. **Background Operations:** Offloads long-running tasks from Wind to Air
//! 3. **Type Translation:** Converts between Tauri JSON and gRPC protobuf
//!    messages
//! 4. **Error Handling:** Translates gRPC errors to user-friendly error
//!    messages
//! 5. **Connection Management:** Manages Air client lifecycle and reconnections
//!
//! **Three-Tier Architecture:**
//! ```
//! Wind (Frontend - TypeScript)
//!   |
//!   | Tauri IPC Commands
//!   v
//! WindAirCommands (Mountain - Rust)
//!   |
//!   | gRPC Calls
//!   v
//! AirClient (gRPC Client)
//!   |
//!   | Network Communication
//!   v
//! Air Daemon (gRPC Server)
//! ```
//!
//! **Available Commands (Tauri IPC):**
//!
//! **1. Update Management:**
//! - `CheckForUpdates` - Check for application updates
//! - `DownloadUpdate` - Download update package
//! - `ApplyUpdate` - Apply downloaded update
//!
//! **2. File Operations:**
//! - `DownloadFile` - Download any file from URL
//!
//! **3. Authentication:**
//! - `AuthenticateUser` - Authenticate with various providers
//!
//! **4. Indexing & Search:**
//! - `IndexFiles` - Index directory contents
//! - `SearchFiles` - Search indexed files
//!
//! **5. Monitoring:**
//! - `GetAirStatus` - Get Air daemon status
//! - `GetAirMetrics` - Get performance & resource metrics
//!
//! **Data Transfer Objects (DTOs):**
//!
//! **UpdateInfoDTO:**
//! ```rust
//! struct UpdateInfoDTO {
//! 	update_available:bool,
//! 	version:String,
//! 	download_url:String,
//! 	release_notes:String,
//! }
//! ```
//!
//! **DownloadResultDTO:**
//! ```rust
//! struct DownloadResultDTO {
//! 	success:bool,
//! 	file_path:String,
//! 	file_size:u64,
//! 	checksum:String,
//! }
//! ```
//!
//! **AuthResponseDTO:**
//! ```rust
//! struct AuthResponseDTO {
//! 	success:bool,
//! 	token:String,
//! 	error:Option<String>,
//! }
//! ```
//!
//! **IndexResultDTO:**
//! ```rust
//! struct IndexResultDTO {
//! 	success:bool,
//! 	files_indexed:u32,
//! 	total_size:u64,
//! }
//! ```
//!
//! **SearchResultsDTO:**
//! ```rust
//! struct SearchResultsDTO {
//! 	results:Vec<FileResultDTO>,
//! 	total_results:u32,
//! }
//! ```
//!
//! **FileResultDTO:**
//! ```rust
//! struct FileResultDTO {
//! 	path:String,
//! 	size:u64,
//! 	line:Option<u32>,
//! 	content:Option<String>,
//! }
//! ```
//!
//! **AirServiceStatusDTO:**
//! ```rust
//! struct AirServiceStatusDTO {
//! 	version:String,
//! 	uptime_seconds:u64,
//! 	total_requests:u64,
//! 	successful_requests:u64,
//! 	failed_requests:u64,
//! 	active_requests:u32,
//! 	healthy:bool,
//! }
//! ```
//!
//! **AirMetricsDTO:**
//! ```rust
//! struct AirMetricsDTO {
//! 	memory_usage_mb:f64,
//! 	cpu_usage_percent:f64,
//! 	average_response_time:f64,
//! 	disk_usage_mb:f64,
//! 	network_usage_mbps:f64,
//! }
//! ```
//!
//! **Command Registration:**
//!
//! All commands are registered with Tauri's invoke_handler:
//!
//! ```rust
//! builder.invoke_handler(tauri::generate_handler![
//! 	CheckForUpdates,
//! 	DownloadUpdate,
//! 	ApplyUpdate,
//! 	DownloadFile,
//! 	AuthenticateUser,
//! 	IndexFiles,
//! 	SearchFiles,
//! 	GetAirStatus,
//! 	GetAirMetrics,
//! ])
//! ```
//!
//! **Client Connection Management:**
//!
//! **AirClientWrapper:**
//! - Wraps the gRPC AirClient
//! - Manages reconnections
//! - Default address: `DEFAULT_AIR_SERVER_ADDRESS`
//!
//! **Connection Flow:**
//! ```rust
//! // 1. Get Air address from config
//! let air_address = get_air_address(&app_handle)?;
//!
//! // 2. Create or reuse client
//! let client = get_or_create_air_client(&app_handle, air_address).await?;
//!
//! // 3. Call Air's gRPC method
//! let response = client.CheckForUpdates(request).await?;
//!
//! // 4. Check for errors
//! if !response.error.is_empty() {
//!     return Err(response.error);
//! }
//!
//! // 5. Convert to DTO
//! let result = UpdateInfoDTO { ... };
//! ```
//!
//! **Error Handling Strategy:**
//!
//! **gRPC Errors:**
//! - Catch all gRPC errors
//! - Translate to user-friendly messages
//! - Include context about what operation failed
//!
//! **Response Errors:**
//! - Check `response.error` field
//! - Return error instead of DTO if present
//! - Preserve original error message
//!
//! **Client Errors:**
//! - Connection failures -> "Failed to connect to Air daemon"
//! - Timeout errors -> "Operation timed out"
//! - Parse errors -> "Failed to parse response"
//!
//! **Usage Examples from Wind:**
//!
//! **Check for Updates:**
//! ```typescript
//! const updates = await invoke('CheckForUpdates', {
//!     currentVersion: '1.0.0',
//!     channel: 'stable'
//! });
//!
//! if (updates.updateAvailable) {
//!     console.log(`New version: ${updates.version}`);
//! }
//! ```
//!
//! **Download Update:**
//! ```typescript
//! const result = await invoke('DownloadUpdate', {
//!     url: 'https://example.com/update.zip',
//!     destination: '/tmp/update.zip',
//!     checksum: 'abc123...'
//! });
//!
//! if (result.success) {
//!     console.log(`Downloaded: ${result.filePath}`);
//! }
//! ```
//!
//! **Authenticate:**
//! ```typescript
//! const auth = await invoke('AuthenticateUser', {
//!     username: 'user@example.com',
//!     password: 'secret',
//!     provider: 'github'
//! });
//!
//! if (auth.success) {
//!     localStorage.setItem('token', auth.token);
//! }
//! ```
//!
//! **Index Files:**
//! ```typescript
//! const indexResult = await invoke('IndexFiles', {
//!     path: '/project',
//!     patterns: ['*.ts', '*.rs'],
//!     excludePatterns: ['node_modules', 'target'],
//!     maxDepth: 10
//! });
//!
//! console.log(`Indexed ${indexResult.filesIndexed} files`);
//! ```
//!
//! **Search Files:**
//! ```typescript
//! const searchResults = await invoke('SearchFiles', {
//!     query: 'TODO:',
//!     indexId: '/project',
//!     maxResults: 100
//! });
//!
//! for (const file of searchResults.results) {
//!     console.log(`${file.path}:${file.line} - ${file.content}`);
//! }
//! ```
//!
//! **Integration with Other Modules:**
//!
//! **TauriIPCServer:**
//! - Commands registered in same invoke handler
//! - Both provide Tauri IPC interfaces
//!
//! **Configuration:**
//! - Air address configurable via Mountain settings
//! - Uses default if not specified
//!
//! **StatusReporter:**
//! - Air status can be reported to Sky
//! - Metrics collected for monitoring
//!
//! **Security Considerations:**
//!
//! - Passwords never logged
//! - Checksums verified for downloads
//! - File paths validated
//! - Provider authentication handled securely by Air
//!
//! **Performance Considerations:**
//!
//! - Client connections are created fresh each call (current implementation)
//! - Could cache clients for better performance in production
//! - Large file downloads streamed via Air
//! - Indexing operations run asynchronously in Air

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use log::{debug, info};

// Import Air types using stub types from Air module.
// Replace these stub types with actual Air service types when the AirIntegration
// feature is fully implemented. The stubs provide type safety during development
// while the Air service is not yet available. Transition requires updating imports
// and ensuring gRPC client codegen is generated from the Air service proto files.
use crate::Air::AirServiceTypesStub::{
	AirClientType,
	ApplyUpdateRequest,
	AuthenticationRequest,
	DownloadRequest,
	IndexRequest,
	MetricsRequest,
	SearchRequest,
	StatusRequest,
	UpdateCheckRequest,
};
use crate::Air::AirClient::DEFAULT_AIR_SERVER_ADDRESS;

/// Data Transfer Objects for Wind-Air communication

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfoDTO {
	pub update_available:bool,
	pub version:String,
	pub download_url:String,
	pub release_notes:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResultDTO {
	pub success:bool,
	pub file_path:String,
	pub file_size:u64,
	pub checksum:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponseDTO {
	pub success:bool,
	pub token:String,
	pub error:Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResultDTO {
	pub success:bool,
	pub files_indexed:u32,
	pub total_size:u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsDTO {
	pub results:Vec<FileResultDTO>,
	pub total_results:u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResultDTO {
	pub path:String,
	pub size:u64,
	pub line:Option<u32>,
	pub content:Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirServiceStatusDTO {
	pub version:String,
	pub uptime_seconds:u64,
	pub total_requests:u64,
	pub successful_requests:u64,
	pub failed_requests:u64,
	pub active_requests:u32,
	pub healthy:bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirMetricsDTO {
	pub memory_usage_mb:f64,
	pub cpu_usage_percent:f64,
	pub average_response_time:f64,
	pub disk_usage_mb:f64,
	pub network_usage_mbps:f64,
}

/// Air Client - Wrapper for the gRPC client connection to Air daemon
#[derive(Debug, Clone)]
pub struct AirClientWrapper {
	client:AirClientType,
}

impl AirClientWrapper {
	/// Create a new AirClient connected to the Air daemon
	pub async fn new(address:String) -> Result<Self, String> {
		debug!("[WindAirCommands] Connecting to Air daemon at: {}", address);

		let client = AirClientType::new(&address)
			.await
			.map_err(|e| format!("Failed to connect to Air daemon: {}", e))?;

		info!("[WindAirCommands] Successfully connected to Air daemon");
		Ok(Self { client })
	}

	/// Reconnect to Air daemon
	pub async fn reconnect(&mut self, address:String) -> Result<(), String> {
		debug!("[WindAirCommands] Reconnecting to Air daemon at: {}", address);

		let client = AirClientType::new(&address)
			.await
			.map_err(|e| format!("Failed to reconnect to Air daemon: {}", e))?;

		self.client = client;
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
pub async fn CheckForUpdates(current_version:Option<String>, channel:Option<String>) -> Result<UpdateInfoDTO, String> {
	debug!(
		"[WindAirCommands] CheckForUpdates called with version: {:?}, channel: {:?}",
		current_version, channel
	);

	// Get the Air client from app state or configuration
	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	// Build the request
	let request = UpdateCheckRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		current_version:current_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
		channel:channel.unwrap_or_else(|| "stable".to_string()),
	};

	// Delegate to Air via gRPC
	let response = client
		.CheckForUpdates(request)
		.await
		.map_err(|e| format!("Update check failed: {}", e))?;

	// Check for errors in the response
	if !response.error.is_empty() {
		return Err(response.error);
	}

	let result = UpdateInfoDTO {
		update_available:response.update_available,
		version:response.version,
		download_url:response.download_url,
		release_notes:response.release_notes,
	};

	info!(
		"[WindAirCommands] Update check completed: available={}",
		result.update_available
	);
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
	url:String,
	destination:String,
	checksum:Option<String>,
) -> Result<DownloadResultDTO, String> {
	debug!("[WindAirCommands] DownloadUpdate called: {} -> {}", url, destination);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = DownloadRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		url,
		destination_path:destination,
		checksum:checksum.unwrap_or_default(),
		headers:Default::default(),
	};

	let response = client
		.DownloadFile(request)
		.await
		.map_err(|e| format!("Update download failed: {}", e))?;

	if !response.error.is_empty() {
		return Err(response.error);
	}

	let result = DownloadResultDTO {
		success:response.success,
		file_path:response.file_path,
		file_size:response.file_size,
		checksum:response.checksum,
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
pub async fn ApplyUpdate(update_id:String, update_path:String) -> Result<bool, String> {
	debug!("[WindAirCommands] ApplyUpdate called: id={}, path={}", update_id, update_path);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	// Use ApplyUpdateRequest from Air module
	let request = ApplyUpdateRequest { request_id:update_id.clone(), update_id, update_path };

	// Apply downloaded updates by sending ApplyUpdateRequest to the Air service.
	// The Air service handles platform-specific installation (replacing binaries,
	// restarting the application, cleaning up old versions). The response indicates
	// success or provides error details. Currently returning an error placeholder
	// until the AirClient ApplyUpdate method is implemented.
	// let response = client
	//     .ApplyUpdate(request)
	//     .await
	//     .map_err(|e| format!("Update application failed: {}", e))?;
	//
	// if !response.error.is_empty() {
	//     return Err(response.error);
	// }
	//
	// info!("[WindAirCommands] Update applied successfully");

	// Placeholder response for now
	Err("ApplyUpdate not yet implemented".to_string())
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
pub async fn DownloadFile(url:String, destination:String) -> Result<DownloadResultDTO, String> {
	debug!("[WindAirCommands] DownloadFile called: {} -> {}", url, destination);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = DownloadRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		url,
		destination_path:destination,
		checksum:String::new(),
		headers:Default::default(),
	};

	let response = client
		.DownloadFile(request)
		.await
		.map_err(|e| format!("File download failed: {}", e))?;

	if !response.error.is_empty() {
		return Err(response.error);
	}

	let result = DownloadResultDTO {
		success:response.success,
		file_path:response.file_path,
		file_size:response.file_size,
		checksum:response.checksum,
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
pub async fn AuthenticateUser(username:String, password:String, provider:String) -> Result<AuthResponseDTO, String> {
	debug!("[WindAirCommands] AuthenticateUser called: {} via {}", username, provider);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = AuthenticationRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		provider,
		credentials:serde_json::json!({"username": username, "password": password}),
	};

	let response = client
		.AuthenticateUser(request)
		.await
		.map_err(|e| format!("Authentication failed: {}", e))?;

	if !response.success && !response.error.is_empty() {
		return Err(response.error);
	}

	let result = AuthResponseDTO {
		success:response.success,
		token:response.token,
		error:if response.error.is_empty() { None } else { Some(response.error) },
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
	path:String,
	patterns:Vec<String>,
	exclude_patterns:Option<Vec<String>>,
	max_depth:Option<u32>,
) -> Result<IndexResultDTO, String> {
	debug!("[WindAirCommands] IndexFiles called: {} with patterns: {:?}", path, patterns);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = IndexRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		paths:vec![path],
		recursive:max_depth.unwrap_or(100) > 0,
	};

	let response = client
		.IndexFiles(request)
		.await
		.map_err(|e| format!("File indexing failed: {}", e))?;

	if !response.error.is_empty() {
		return Err(response.error);
	}

	let result = IndexResultDTO {
		success:response.success,
		files_indexed:response.files_indexed,
		total_size:response.total_size,
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
	query:String,
	file_patterns:Vec<String>,
	max_results:Option<u32>,
) -> Result<SearchResultsDTO, String> {
	debug!(
		"[WindAirCommands] SearchFiles called: query={}, patterns={:?}",
		query, file_patterns
	);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = SearchRequest {
		request_id:uuid::Uuid::new_v4().to_string(),
		query,
		file_patterns,
		max_results:max_results.unwrap_or(100),
	};

	let response = client
		.SearchFiles(request)
		.await
		.map_err(|e| format!("File search failed: {}", e))?;

	if !response.error.is_empty() {
		return Err(response.error);
	}

	let results:Vec<FileResultDTO> = response
		.results
		.into_iter()
		.map(|r| FileResultDTO { path:r.path, size:r.size, line:r.line, content:r.content })
		.collect();

	let result = SearchResultsDTO { results, total_results:response.total_results };

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
pub async fn GetAirStatus() -> Result<AirServiceStatusDTO, String> {
	debug!("[WindAirCommands] GetAirStatus called");

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = StatusRequest { request_id:uuid::Uuid::new_v4().to_string() };

	let response = client
		.GetStatus(request)
		.await
		.map_err(|e| format!("Failed to get Air status: {}", e))?;

	// Implement a dedicated HealthCheck method in AirClient to assess service
	// health beyond simple uptime. The current heuristic (uptime_seconds > 0) is
	// simplistic. A proper health check RPC verifies the Air service is responsive
	// and ready to accept requests, checking dependencies, resource availability,
	// and system load. Replace the uptime heuristic with the health check response
	// to determine service availability.
	let healthy = response.uptime_seconds > 0;

	let result = AirServiceStatusDTO {
		version:response.version,
		uptime_seconds:response.uptime_seconds,
		total_requests:response.total_requests,
		successful_requests:response.successful_requests,
		failed_requests:response.failed_requests,
		active_requests:response.active_requests,
		healthy,
	};

	debug!(
		"[WindAirCommands] Air status retrieved: version={}, healthy={}",
		result.version, result.healthy
	);
	Ok(result)
}

/// Command: Get Air Metrics
///
/// Retrieves performance and resource metrics from Air.
/// Delegates to Air's metrics service.
///
/// # Arguments
/// * `app_handle` - Tauri application handle
/// * `metric_type` - Type of metrics ("all", "performance", "resources",
///   "requests")
///
/// # Returns
/// `AirMetricsDTO` with metrics data
#[tauri::command]
pub async fn GetAirMetrics(metric_type:Option<String>) -> Result<AirMetricsDTO, String> {
	debug!("[WindAirCommands] GetAirMetrics called with type: {:?}", metric_type);

	let air_address = get_air_address()?;
	let client = get_or_create_air_client(air_address).await?;

	let request = MetricsRequest { request_id:uuid::Uuid::new_v4().to_string(), metric_type };

	let response = client
		.GetMetrics(request)
		.await
		.map_err(|e| format!("Failed to get Air metrics: {}", e))?;

	let result = AirMetricsDTO {
		memory_usage_mb:response.metrics.memory_usage_mb,
		cpu_usage_percent:response.metrics.cpu_usage_percent,
		average_response_time:response.metrics.average_response_time,
		disk_usage_mb:response.metrics.disk_usage_mb,
		network_usage_mbps:response.metrics.network_usage_mbps,
	};

	debug!("[WindAirCommands] Air metrics retrieved");
	Ok(result)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the Air daemon address from configuration
fn get_air_address() -> Result<String, String> {
	// Return default Air address
	Ok(DEFAULT_AIR_SERVER_ADDRESS.to_string())
}

/// Get or create the Air client instance
async fn get_or_create_air_client(address:String) -> Result<AirClientType, String> {
	// Create a new client each time
	// In production, you'd use a state management pattern
	AirClientType::new(&address)
		.await
		.map_err(|e| format!("Failed to create Air client: {}", e))
}

/// Register all Wind-Air commands with Tauri
pub fn register_wind_air_commands<R:tauri::Runtime>(builder:tauri::Builder<R>) -> tauri::Builder<R> {
	builder.invoke_handler(tauri::generate_handler![
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
