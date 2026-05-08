#![allow(non_snake_case)]

//! # AirClient
//!
//! gRPC client wrapper for the Air daemon service. Mountain reaches Air
//! through this façade for update management, authentication, file
//! indexing, and system monitoring. Companion DTOs live in sibling
//! files declared below; the streaming helper lives in
//! `DownloadStream::Struct`.

pub mod AirMetrics;

pub mod AirStatus;

pub mod DownloadStream;

pub mod DownloadStreamChunk;

pub mod ExtendedFileInfo;

pub mod FileInfo;

pub mod FileResult;

pub mod IndexInfo;

pub mod ResourceUsage;

pub mod UpdateInfo;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use CommonLibrary::Error::CommonError::CommonError;
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use tonic::{Request, transport::Channel};

use crate::dev_log;

/// Default gRPC server address for the Air daemon.
///
/// Port Allocation:
/// - 50051: Mountain Vine server
/// - 50052: Cocoon Vine server (VS Code extension hosting)
/// - 50053: Air Vine server (Air daemon services - authentication, updates, and
///   more)
pub const DEFAULT_AIR_SERVER_ADDRESS:&str = "[::1]:50053";

/// Air gRPC client wrapper that handles connection to the Air daemon service.
/// This provides a clean interface for Mountain to interact with Air's
/// capabilities including update management, authentication, file indexing,
/// and system monitoring.
#[derive(Clone)]
pub struct AirClient {
	#[cfg(feature = "AirIntegration")]
	/// The underlying tonic gRPC client wrapped in Arc<Mutex<>> for thread-safe
	/// access
	client:Option<Arc<Mutex<AirServiceClient<Channel>>>>,

	/// Address of the Air daemon
	address:String,
}

impl AirClient {
	/// Creates a new AirClient and connects to the Air daemon service.
	///
	/// # Arguments
	/// * `address` - The gRPC server address (e.g., "http://\\[::1\\]:50053")
	///
	/// # Returns
	/// * `Ok(Self)` - Successfully created client
	/// * `Err(CommonError)` - Connection failure with descriptive error
	///
	/// # Example
	///
	/// ```text
	/// use Mountain::Air::AirClient::{AirClient, DEFAULT_AIR_SERVER_ADDRESS};
	///
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let client = AirClient::new(DEFAULT_AIR_SERVER_ADDRESS).await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn new(address:&str) -> Result<Self, CommonError> {
		dev_log!("grpc", "[AirClient] Connecting to Air daemon at: {}", address);

		#[cfg(feature = "AirIntegration")]
		{
			let endpoint = address.parse::<tonic::transport::Endpoint>().map_err(|e| {
				dev_log!("grpc", "error: [AirClient] Failed to parse address '{}': {}", address, e);
				CommonError::IPCError { Description:format!("Invalid address '{}': {}", address, e) }
			})?;

			let channel = endpoint.connect().await.map_err(|e| {
				dev_log!("grpc", "error: [AirClient] Failed to connect to Air daemon: {}", e);
				CommonError::IPCError { Description:format!("Connection failed: {}", e) }
			})?;

			dev_log!("grpc", "[AirClient] Successfully connected to Air daemon at: {}", address);

			let client = Arc::new(Mutex::new(AirServiceClient::new(channel)));

			Ok(Self { client:Some(client), address:address.to_string() })
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			dev_log!("grpc", "error: [AirClient] AirIntegration feature is not enabled");

			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Checks if the client is connected to the Air daemon.
	///
	/// # Returns
	/// * `true` - Client is connected
	/// * `false` - Client is not connected
	pub fn is_connected(&self) -> bool {
		#[cfg(feature = "AirIntegration")]
		{
			self.client.is_some()
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			false
		}
	}

	/// Gets the address of the Air daemon.
	///
	/// # Returns
	/// The address string
	pub fn address(&self) -> &str { &self.address }

	// =========================================================================
	// Authentication Operations
	// =========================================================================

	/// Authenticates a user with the Air daemon.
	///
	/// # Arguments
	/// * `username` - User's username
	/// * `password` - User's password
	/// * `provider` - Authentication provider (e.g., "github", "gitlab",
	///   "microsoft")
	///
	/// # Returns
	/// * `Ok(token)` - Authentication token if successful
	/// * `Err(CommonError)` - Authentication failure
	pub async fn authenticate(
		&self,

		request_id:String,

		username:String,

		password:String,

		provider:String,
	) -> Result<String, CommonError> {
		dev_log!(
			"grpc",
			"[AirClient] Authenticating user '{}' with provider '{}'",
			username,
			provider
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::AuthenticationRequest;

			let username_display = username.clone();

			let request = AuthenticationRequest { request_id, username, password, provider };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.authenticate(Request::new(request)).await {
				Ok(response) => {
					let response = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Authentication successful for user '{}'", username_display);

						Ok(response.token)
					} else {
						dev_log!(
							"grpc",
							"error: [AirClient] Authentication failed for user '{}': {}",
							username_display,
							response.error
						);

						Err(CommonError::AccessDenied { Reason:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Authentication RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Authentication RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// Update Operations
	// =========================================================================

	/// Checks for available updates.
	///
	/// # Arguments
	/// * `current_version` - Current application version
	/// * `channel` - Update channel (e.g., "stable", "beta", "nightly")
	///
	/// # Returns
	/// * `Ok(update_info)` - Update information if available
	/// * `Err(CommonError)` - Check failure
	pub async fn check_for_updates(
		&self,

		request_id:String,

		current_version:String,

		channel:String,
	) -> Result<UpdateInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Checking for updates for version '{}'", current_version);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::UpdateCheckRequest;

			let request = UpdateCheckRequest { request_id, current_version, channel };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.check_for_updates(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::UpdateCheckResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Update check completed. Update available: {}",
						response.update_available
					);

					Ok(UpdateInfo::Struct {
						update_available:response.update_available,
						version:response.version,
						download_url:response.download_url,
						release_notes:response.release_notes,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Check for updates RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Check for updates RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Downloads an update package.
	///
	/// # Arguments
	/// * `url` - URL of the update package
	/// * `destination_path` - Local path to save the downloaded file
	/// * `checksum` - Optional SHA256 checksum for verification
	/// * `headers` - Optional HTTP headers
	///
	/// # Returns
	/// * `Ok(file_info)` - Downloaded file information
	/// * `Err(CommonError)` - Download failure
	pub async fn download_update(
		&self,

		request_id:String,

		url:String,

		destination_path:String,

		checksum:String,

		headers:HashMap<String, String>,
	) -> Result<FileInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Downloading update from: {}", url);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::DownloadRequest;

			let request = DownloadRequest { request_id, url, destination_path, checksum, headers };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.download_update(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::DownloadResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Update downloaded successfully to: {}", response.file_path);

						Ok(FileInfo::Struct {
							file_path:response.file_path,
							file_size:response.file_size,
							checksum:response.checksum,
						})
					} else {
						dev_log!("grpc", "error: [AirClient] Update download failed: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Download update RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Download update RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Applies an update package.
	///
	/// # Arguments
	/// * `version` - Version of the update
	/// * `update_path` - Path to the update package
	///
	/// # Returns
	/// * `Ok(())` - Update applied successfully
	/// * `Err(CommonError)` - Application failure
	pub async fn apply_update(&self, request_id:String, version:String, update_path:String) -> Result<(), CommonError> {
		dev_log!("grpc", "[AirClient] Applying update version: {}", version);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ApplyUpdateRequest;

			let request = ApplyUpdateRequest { request_id, version, update_path };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.apply_update(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::ApplyUpdateResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Update applied successfully");

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Update application failed: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Apply update RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Apply update RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// Download Operations
	// =========================================================================

	/// Downloads a file.
	///
	/// # Arguments
	/// * `url` - URL of the file to download
	/// * `destination_path` - Local path to save the downloaded file
	/// * `checksum` - Optional SHA256 checksum for verification
	/// * `headers` - Optional HTTP headers
	///
	/// # Returns
	/// * `Ok(file_info)` - Downloaded file information
	/// * `Err(CommonError)` - Download failure
	pub async fn download_file(
		&self,

		request_id:String,

		url:String,

		destination_path:String,

		checksum:String,

		headers:HashMap<String, String>,
	) -> Result<FileInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Downloading file from: {}", url);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::DownloadRequest;

			let request = DownloadRequest { request_id, url, destination_path, checksum, headers };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.download_file(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::DownloadResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] File downloaded successfully to: {}", response.file_path);

						Ok(FileInfo::Struct {
							file_path:response.file_path,
							file_size:response.file_size,
							checksum:response.checksum,
						})
					} else {
						dev_log!("grpc", "error: [AirClient] File download failed: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Download file RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Download file RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Downloads a file as a stream.
	///
	/// This method initiates a streaming download from the given URL, returning
	/// a stream of chunks that can be processed incrementally without loading
	/// the entire file into memory.
	///
	/// # Arguments
	/// * `request_id` - Unique request identifier
	/// * `url` - URL of the file to download
	/// * `headers` - Optional HTTP headers
	///
	/// # Returns
	/// * `Ok(stream)` - Stream that yields download chunks
	/// * `Err(CommonError)` - Download initiation failure
	///
	/// # Stream Chunk Information
	///
	/// Each chunk contains:
	/// - `chunk`: The binary data chunk
	/// - `total_size`: Total file size (if known)
	/// - `downloaded`: Number of bytes downloaded so far
	/// - `completed`: Whether this is the final chunk
	/// - `error`: Error message if download failed
	///
	/// # Example
	///
	/// ```text
	/// use Mountain::Air::AirClient::AirClient;
	/// use CommonLibrary::Error::CommonError::CommonError;
	///
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let client = AirClient::new("http://[::1]:50053").await?;
	/// let mut stream = client
	/// 	.download_stream(
	/// 		"req-123".to_string(),
	/// 		"https://example.com/large-file.zip".to_string(),
	/// 		std::collections::HashMap::new(),
	/// 	)
	/// 	.await?;
	///
	/// let mut buffer = Vec::new();
	/// while let Some(chunk) = stream.next().await {
	/// 	let chunk = chunk?;
	/// 	buffer.extend_from_slice(&chunk.data);
	/// 	println!("Downloaded: {} / {} bytes", chunk.downloaded, chunk.total_size);
	/// 	if chunk.completed {
	/// 		break;
	/// 	}
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub async fn download_stream(
		&self,

		request_id:String,

		url:String,

		headers:HashMap<String, String>,
	) -> Result<DownloadStream::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Starting stream download from: {}", url);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::DownloadStreamRequest;

			let request = DownloadStreamRequest { request_id, url, headers };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.download_stream(Request::new(request)).await {
				Ok(response) => {
					dev_log!("grpc", "[AirClient] Stream download initiated successfully");

					Ok(DownloadStream::Struct::new(response.into_inner()))
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Download stream RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Download stream RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// File Indexing Operations
	// =========================================================================

	/// Indexes files in a directory.
	///
	/// # Arguments
	/// * `path` - Path to the directory to index
	/// * `patterns` - File patterns to include
	/// * `exclude_patterns` - File patterns to exclude
	/// * `max_depth` - Maximum depth for recursion
	///
	/// # Returns
	/// * `Ok(index_info)` - Index information
	/// * `Err(CommonError)` - Indexing failure
	pub async fn index_files(
		&self,

		request_id:String,

		path:String,

		patterns:Vec<String>,

		exclude_patterns:Vec<String>,

		max_depth:u32,
	) -> Result<IndexInfo::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Indexing files in: {}", path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::IndexRequest;

			let request = IndexRequest { request_id, path, patterns, exclude_patterns, max_depth };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.index_files(Request::new(request)).await {
				Ok(response) => {
					let response = response.into_inner();

					// Use fields that actually exist in IndexResponse
					dev_log!(
						"grpc",
						"[AirClient] Files indexed: {} (total size: {} bytes)",
						response.files_indexed,
						response.total_size
					);

					Ok(IndexInfo::Struct { files_indexed:response.files_indexed, total_size:response.total_size })
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Index files RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Index files RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Searches for files matching a query.
	///
	/// # Arguments
	/// * `query` - Search query string
	/// * `path` - Path to search in
	/// * `max_results` - Maximum number of results to return
	///
	/// # Returns
	/// * `Ok(results)` - Search results
	/// * `Err(CommonError)` - Search failure
	pub async fn search_files(
		&self,

		request_id:String,

		query:String,

		path:String,

		max_results:u32,
	) -> Result<Vec<FileResult::Struct>, CommonError> {
		dev_log!("grpc", "[AirClient] Searching for files with query: '{}' in: {}", query, path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::SearchRequest;

			let request = SearchRequest { request_id, query, path, max_results };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.search_files(Request::new(request)).await {
				Ok(_response) => {
					dev_log!("grpc", "[AirClient] Search completed");

					// Placeholder implementation - actual response structure may vary
					Ok(Vec::new())
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Search files RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Search files RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Gets file information.
	///
	/// # Arguments
	/// * `path` - Path to the file
	///
	/// # Returns
	/// * `Ok(file_info)` - File information
	/// * `Err(CommonError)` - Request failure
	pub async fn get_file_info(&self, request_id:String, path:String) -> Result<ExtendedFileInfo::Struct, CommonError> {
		let path_display = path.clone();

		dev_log!("grpc", "[AirClient] Getting file info for: {}", path);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::FileInfoRequest;

			let request = FileInfoRequest { request_id, path };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.get_file_info(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::FileInfoResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] File info retrieved for: {} (exists: {})",
						path_display,
						response.exists
					);

					Ok(ExtendedFileInfo::Struct {
						exists:response.exists,
						size:response.size,
						mime_type:response.mime_type,
						checksum:response.checksum,
						modified_time:response.modified_time,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get file info RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get file info RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// Status and Monitoring Operations
	// =========================================================================

	/// Gets the status of the Air daemon.
	///
	/// # Returns
	/// * `Ok(status)` - Air daemon status
	/// * `Err(CommonError)` - Request failure
	pub async fn get_status(&self, request_id:String) -> Result<AirStatus::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting Air daemon status");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::StatusRequest;

			let request = StatusRequest { request_id };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.get_status(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::StatusResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Status retrieved. Active requests: {}",
						response.active_requests
					);

					Ok(AirStatus::Struct {
						version:response.version,
						uptime_seconds:response.uptime_seconds,
						total_requests:response.total_requests,
						successful_requests:response.successful_requests,
						failed_requests:response.failed_requests,
						average_response_time:response.average_response_time,
						memory_usage_mb:response.memory_usage_mb,
						cpu_usage_percent:response.cpu_usage_percent,
						active_requests:response.active_requests,
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get status RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get status RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Performs a health check on the Air daemon.
	///
	/// # Returns
	/// * `Ok(healthy)` - Health status
	/// * `Err(CommonError)` - Check failure
	pub async fn health_check(&self) -> Result<bool, CommonError> {
		dev_log!("grpc", "[AirClient] Performing health check");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::HealthCheckRequest;

			let request = HealthCheckRequest {};

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.health_check(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::HealthCheckResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Health check result: {}", response.healthy);

					Ok(response.healthy)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Health check RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Health check RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			// When AirIntegration is not enabled, we return true to allow
			// the application to function without Air
			Ok(true)
		}
	}

	/// Gets metrics from the Air daemon.
	///
	/// # Arguments
	/// * `metric_type` - Type of metrics (e.g., "performance", "resources",
	///   "requests")
	///
	/// # Returns
	/// * `Ok(metrics)` - Metrics data
	/// * `Err(CommonError)` - Request failure
	pub async fn get_metrics(
		&self,

		request_id:String,

		metric_type:Option<String>,
	) -> Result<AirMetrics::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting metrics (type: {:?})", metric_type.as_deref());

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::MetricsRequest;

			let request = MetricsRequest { request_id, metric_type:metric_type.unwrap_or_default() };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.get_metrics(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::MetricsResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Metrics retrieved");

					// Parse metrics from the string map - this is a simplified implementation
					let metrics = AirMetrics::Struct {
						memory_usage_mb:response
							.metrics
							.get("memory_usage_mb")
							.and_then(|s| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						cpu_usage_percent:response
							.metrics
							.get("cpu_usage_percent")
							.and_then(|s| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						network_usage_mbps:response
							.metrics
							.get("network_usage_mbps")
							.and_then(|s| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						disk_usage_mb:response
							.metrics
							.get("disk_usage_mb")
							.and_then(|s| s.parse::<f64>().ok())
							.unwrap_or(0.0),

						average_response_time:response
							.metrics
							.get("average_response_time")
							.and_then(|s| s.parse::<f64>().ok())
							.unwrap_or(0.0),
					};

					Ok(metrics)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get metrics RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get metrics RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// Resource Management Operations
	// =========================================================================

	/// Gets resource usage information.
	///
	/// # Arguments
	/// * `request_id` - Unique request identifier
	///
	/// # Returns
	/// * `Ok(usage)` - Resource usage data
	/// * `Err(CommonError)` - Request failure
	pub async fn get_resource_usage(&self, request_id:String) -> Result<ResourceUsage::Struct, CommonError> {
		dev_log!("grpc", "[AirClient] Getting resource usage");

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ResourceUsageRequest;

			let request = ResourceUsageRequest { request_id };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.get_resource_usage(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::ResourceUsageResponse = response.into_inner();

					dev_log!("grpc", "[AirClient] Resource usage retrieved");

					Ok(ResourceUsage::Struct {
						memory_usage_mb:response.memory_usage_mb,
						cpu_usage_percent:response.cpu_usage_percent,
						disk_usage_mb:response.disk_usage_mb,
						network_usage_mbps:response.network_usage_mbps,
						thread_count:0,      // Not provided in ResourceUsageResponse
						open_file_handles:0, // Not provided in ResourceUsageResponse
					})
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get resource usage RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get resource usage RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Sets resource limits.
	///
	/// # Arguments
	/// * `request_id` - Unique request identifier
	/// * `memory_limit_mb` - Memory limit in MB
	/// * `cpu_limit_percent` - CPU limit as percentage
	/// * `disk_limit_mb` - Disk limit in MB
	///
	/// # Returns
	/// * `Ok(())` - Limits set successfully
	/// * `Err(CommonError)` - Set failure
	pub async fn set_resource_limits(
		&self,

		request_id:String,

		memory_limit_mb:u32,

		cpu_limit_percent:u32,

		disk_limit_mb:u32,
	) -> Result<(), CommonError> {
		dev_log!(
			"grpc",
			"[AirClient] Setting resource limits: memory={}MB, cpu={}%, disk={}MB",
			memory_limit_mb,
			cpu_limit_percent,
			disk_limit_mb
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ResourceLimitsRequest;

			let request = ResourceLimitsRequest { request_id, memory_limit_mb, cpu_limit_percent, disk_limit_mb };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.set_resource_limits(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::ResourceLimitsResponse = response.into_inner();

					if response.success {
						dev_log!("grpc", "[AirClient] Resource limits set successfully");

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Failed to set resource limits: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Set resource limits RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Set resource limits RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	// =========================================================================
	// Configuration Management Operations
	// =========================================================================

	/// Gets configuration.
	///
	/// # Arguments
	/// * `section` - Configuration section (e.g., "grpc", "authentication",
	///   "updates")
	///
	/// # Returns
	/// * `Ok(config)` - Configuration data
	/// * `Err(CommonError)` - Request failure
	pub async fn get_configuration(
		&self,

		request_id:String,

		section:String,
	) -> Result<HashMap<String, String>, CommonError> {
		let section_display = section.clone();

		dev_log!("grpc", "[AirClient] Getting configuration for section: {}", section);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::ConfigurationRequest;

			let request = ConfigurationRequest { request_id, section };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.get_configuration(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::ConfigurationResponse = response.into_inner();

					dev_log!(
						"grpc",
						"[AirClient] Configuration retrieved for section: {} ({} keys)",
						section_display,
						response.configuration.len()
					);

					Ok(response.configuration)
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Get configuration RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Get configuration RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}

	/// Updates configuration.
	///
	/// # Arguments
	/// * `section` - Configuration section
	/// * `updates` - Configuration updates
	///
	/// # Returns
	/// * `Ok(())` - Configuration updated successfully
	/// * `Err(CommonError)` - Update failure
	pub async fn update_configuration(
		&self,

		request_id:String,

		section:String,

		updates:HashMap<String, String>,
	) -> Result<(), CommonError> {
		let section_display = section.clone();

		dev_log!(
			"grpc",
			"[AirClient] Updating configuration for section: {} ({} keys)",
			section_display,
			updates.len()
		);

		#[cfg(feature = "AirIntegration")]
		{
			use AirLibrary::Vine::Generated::air::UpdateConfigurationRequest;

			let request = UpdateConfigurationRequest { request_id, section, updates };

			let client = self
				.client
				.as_ref()
				.ok_or_else(|| CommonError::IPCError { Description:"Air client not initialized".to_string() })?;

			let mut client_guard = client.lock().await;

			match client_guard.update_configuration(Request::new(request)).await {
				Ok(response) => {
					let response:AirLibrary::Vine::Generated::air::UpdateConfigurationResponse = response.into_inner();

					if response.success {
						dev_log!(
							"grpc",
							"[AirClient] Configuration updated successfully for section: {}",
							section_display
						);

						Ok(())
					} else {
						dev_log!("grpc", "error: [AirClient] Failed to update configuration: {}", response.error);

						Err(CommonError::IPCError { Description:response.error })
					}
				},

				Err(e) => {
					dev_log!("grpc", "error: [AirClient] Update configuration RPC error: {}", e);

					Err(CommonError::IPCError { Description:format!("Update configuration RPC error: {}", e) })
				},
			}
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Err(CommonError::FeatureNotAvailable { FeatureName:"AirIntegration".to_string() })
		}
	}
}

// ============================================================================
// Response Types
// ============================================================================
// ============================================================================
// Debug Implementation
// ============================================================================

impl std::fmt::Debug for AirClient {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "AirClient({})", self.address) }
}

// ============================================================================
// tonic::Request Helper
// ============================================================================

/// Helper trait for converting types to tonic::Request
#[allow(dead_code)]
trait IntoRequestExt {
	fn into_request(self) -> tonic::Request<Self>
	where
		Self: Sized, {
		tonic::Request::new(self)
	}
}

impl<T> IntoRequestExt for T {}
