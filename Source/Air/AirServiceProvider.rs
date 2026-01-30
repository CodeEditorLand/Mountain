// File: Mountain/Source/Air/AirServiceProvider.rs
// Role: High-level API surface for Air service methods
// Responsibilities:
//   - Provide a cleaner, high-level interface to AirClient
//   - Add convenience methods that delegate to AirClient
//   - Handle common patterns like request ID generation
//   - Provide consistent error handling across all Air operations

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::Error::CommonError::CommonError;
use log::debug;
use uuid::Uuid;

use super::AirClient::*;
use super::AirClient::{
    AuthenticationRequest, AuthenticationResponse,
    UpdateCheckRequest, UpdateCheckResponse,
    DownloadRequest, DownloadResponse,
    IndexRequest, IndexResponse,
    SearchRequest, SearchResponse,
    StatusRequest, StatusResponse,
    MetricsRequest, MetricsResponse,
};

/// AirServiceProvider provides a high-level, convenient interface to the Air daemon service.
/// This struct wraps the lower-level AirClient and adds convenience methods,
/// automatic request ID generation, and enhanced error handling.
///
/// All methods delegate to the underlying AirClient but provide a cleaner API
/// for use throughout the Mountain application.
#[derive(Clone)]
pub struct AirServiceProvider {
	client: Arc<AirClient>,
}

impl AirServiceProvider {
	/// Creates a new AirServiceProvider with the given AirClient.
	///
	/// # Arguments
	/// * `client` - The AirClient to wrap
	///
	/// # Returns
	/// A new AirServiceProvider instance
	pub fn new(client: Arc<AirClient>) -> Self {
		Self { client }
	}

	/// Gets a reference to the underlying AirClient.
	pub fn client(&self) -> &Arc<AirClient> {
		&self.client
	}

	/// Checks if Air is available and connected.
	pub fn is_available(&self) -> bool {
		self.client.is_connected()
	}

	// =========================================================================
	// High-Level Update Operations
	// =========================================================================

	/// Checks for available updates for the application with automatic request ID generation.
	///
	/// # Arguments
	/// * `current_version` - The current application version
	/// * `channel` - The update channel ("stable", "beta", or "nightly")
	///
	/// # Returns
	/// Response containing available update information
	pub async fn CheckForUpdates(
		&self,
		current_version: &str,
		channel: &str,
	) -> Result<UpdateCheckResponse, CommonError> {
		let request_id = generate_request_id();
		debug!(
			"[AirServiceProvider] CheckForUpdates: current_version={}, channel={}",
			current_version, channel
		);

		let request = UpdateCheckRequest {
			request_id,
			current_version: current_version.to_string(),
			channel: channel.to_string(),
		};

		self.client.CheckForUpdates(request).await
	}

	/// Downloads a file from a URL with automatic request ID generation.
	///
	/// # Arguments
	/// * `url` - The URL to download from
	/// * `destination_path` - Local path to save the downloaded file
	/// * `checksum` - Optional SHA256 checksum for verification
	/// * `headers` - Optional HTTP headers to include in the request
	///
	/// # Returns
	/// Response containing download result
	pub async fn DownloadFile(
		&self,
		url: &str,
		destination_path: &str,
		checksum: Option<&str>,
		headers: Option<std::collections::HashMap<String, String>>,
	) -> Result<DownloadResponse, CommonError> {
		let request_id = generate_request_id();
		debug!("[AirServiceProvider] DownloadFile: url={}, destination={}", url, destination_path);

		let request = DownloadRequest {
			request_id,
			url: url.to_string(),
			destination_path: destination_path.to_string(),
			checksum: checksum.unwrap_or("").to_string(),
			headers: headers.unwrap_or_default(),
		};

		self.client.DownloadFile(request).await
	}

	// =========================================================================
	// High-Level Authentication Operations
	// =========================================================================

	/// Authenticates a user with the specified credentials and provider.
	///
	/// # Arguments
	/// * `username` - The username or identifier
	/// * `password` - The password or token
	/// * `provider` - The authentication provider (e.g., "github", "gitlab")
	///
	/// # Returns
	/// Response containing authentication token and success status
	pub async fn Authenticate(
		&self,
		username: &str,
		password: &str,
		provider: &str,
	) -> Result<AuthenticationResponse, CommonError> {
		let request_id = generate_request_id();
		debug!("[AirServiceProvider] Authenticate: username={}, provider={}", username, provider);

		let request = AuthenticationRequest {
			request_id,
			username: username.to_string(),
			password: password.to_string(),
			provider: provider.to_string(),
		};

		self.client.Authenticate(request).await
	}

	// =========================================================================
	// High-Level File Indexing Operations
	// =========================================================================

	/// Indexes files in the specified path for search functionality.
	///
	/// # Arguments
	/// * `path` - The directory path to index
	/// * `patterns` - File patterns to include (e.g., ["*.rs", "*.ts"])
	/// * `exclude_patterns` - File patterns to exclude
	/// * `max_depth` - Maximum directory depth to traverse (0 for unlimited)
	///
	/// # Returns
	/// Response containing indexing statistics
	pub async fn IndexFiles(
		&self,
		path: &str,
		patterns: &[String],
		exclude_patterns: &[String],
		max_depth: u32,
	) -> Result<IndexResponse, CommonError> {
		let request_id = generate_request_id();
		debug!(
			"[AirServiceProvider] IndexFiles: path={}, patterns={}, exclude={}, depth={}",
			path,
			patterns.len(),
			exclude_patterns.len(),
			max_depth
		);

		let request = IndexRequest {
			request_id,
			path: path.to_string(),
			patterns: patterns.to_vec(),
			exclude_patterns: exclude_patterns.to_vec(),
			max_depth,
		};

		self.client.IndexFiles(request).await
	}

	/// Searches indexed files for the specified query.
	///
	/// # Arguments
	/// * `query` - The search query string
	/// * `path` - The directory path to search within (optional)
	/// * `max_results` - Maximum number of results to return
	///
	/// # Returns
	/// Response containing search results
	pub async fn SearchFiles(
		&self,
		query: &str,
		path: Option<&str>,
		max_results: u32,
	) -> Result<SearchResponse, CommonError> {
		let request_id = generate_request_id();
		debug!(
			"[AirServiceProvider] SearchFiles: query={}, path={}, max_results={}",
			query,
			path.unwrap_or("<all>"),
			max_results
		);

		let request = SearchRequest {
			request_id,
			query: query.to_string(),
			path: path.unwrap_or("").to_string(),
			max_results,
		};

		self.client.SearchFiles(request).await
	}

	// =========================================================================
	// High-Level Status and Monitoring Operations
	// =========================================================================

	/// Gets the current status of the Air daemon.
	///
	/// # Returns
	/// Response containing system status metrics
	pub async fn GetStatus(&self) -> Result<StatusResponse, CommonError> {
		let request_id = generate_request_id();
		debug!("[AirServiceProvider] GetStatus");

		let request = StatusRequest { request_id };

		self.client.GetStatus(request).await
	}

	/// Gets detailed metrics from the Air daemon.
	///
	/// # Arguments
	/// * `metric_type` - The type of metrics to retrieve
	///                   ("performance", "resources", "requests")
	///
	/// # Returns
	/// Response containing requested metrics
	pub async fn GetMetrics(&self, metric_type: &str) -> Result<MetricsResponse, CommonError> {
		let request_id = generate_request_id();
		debug!("[AirServiceProvider] GetMetrics: type={}", metric_type);

		let request = MetricsRequest {
			request_id,
			metric_type: metric_type.to_string(),
		};

		self.client.GetMetrics(request).await
	}

	// =========================================================================
	// Convenience Methods for Common Operations
	// =========================================================================

	/// Checks for stable channel updates.
	///
	/// # Arguments
	/// * `current_version` - The current application version
	///
	/// # Returns
	/// Response containing available update information
	pub async fn CheckForStableUpdate(
		&self,
		current_version: &str,
	) -> Result<UpdateCheckResponse, CommonError> {
		self.CheckForUpdates(current_version, "stable").await
	}

	/// Checks for beta channel updates.
	///
	/// # Arguments
	/// * `current_version` - The current application version
	///
	/// # Returns
	/// Response containing available update information
	pub async fn CheckForBetaUpdate(&self, current_version: &str) -> Result<UpdateCheckResponse, CommonError> {
		self.CheckForUpdates(current_version, "beta").await
	}

	/// Downloads a simple file without checksum verification.
	///
	/// # Arguments
	/// * `url` - The URL to download from
	/// * `destination_path` - Local path to save the downloaded file
	///
	/// # Returns
	/// Response containing download result
	pub async fn DownloadSimpleFile(&self, url: &str, destination_path: &str) -> Result<DownloadResponse, CommonError> {
		self.DownloadFile(url, destination_path, None, None).await
	}

	/// Indexes source code files with common patterns.
	///
	/// # Arguments
	/// * `path` - The directory path to index
	/// * `max_depth` - Maximum directory depth to traverse
	///
	/// # Returns
	/// Response containing indexing statistics
	pub async fn IndexSourceFiles(
		&self,
		path: &str,
		max_depth: u32,
	) -> Result<IndexResponse, CommonError> {
		let patterns = vec![
			"*.rs".to_string(),
			"*.ts".to_string(),
			"*.tsx".to_string(),
			"*.js".to_string(),
			"*.jsx".to_string(),
			"*.py".to_string(),
			"*.go".to_string(),
			"*.java".to_string(),
			"*.c".to_string(),
			"*.cpp".to_string(),
			"*.h".to_string(),
			"*.hpp".to_string(),
		];

		let exclude_patterns = vec!["node_modules/**".to_string(), "target/**".to_string(), ".git/**".to_string()];

		self.IndexFiles(path, &patterns, &exclude_patterns, max_depth).await
	}

	/// Searches all indexed files.
	///
	/// # Arguments
	/// * `query` - The search query string
	/// * `max_results` - Maximum number of results to return
	///
	/// # Returns
	/// Response containing search results
	pub async fn SearchAll(&self, query: &str, max_results: u32) -> Result<SearchResponse, CommonError> {
		self.SearchFiles(query, None, max_results).await
	}

	/// Gets performance metrics.
	///
	/// # Returns
	/// Response containing performance metrics
	pub async fn GetPerformanceMetrics(&self) -> Result<MetricsResponse, CommonError> {
		self.GetMetrics("performance").await
	}

	/// Gets resource usage metrics.
	///
	/// # Returns
	/// Response containing resource usage metrics
	pub async fn GetResourceMetrics(&self) -> Result<MetricsResponse, CommonError> {
		self.GetMetrics("resources").await
	}

	/// Gets request statistics.
	///
	/// # Returns
	/// Response containing request statistics
	pub async fn GetRequestMetrics(&self) -> Result<MetricsResponse, CommonError> {
		self.GetMetrics("requests").await
	}

	/// Checks if Air is healthy and responsive.
	///
	/// This is a convenience method that calls GetStatus and checks if the
	/// response indicates a healthy state.
	///
	/// # Returns
	/// * `Ok(true)` - Air is available and healthy
	/// * `Ok(false)` - Air is available but not healthy
	/// * `Err(CommonError)` - Air is unavailable or request failed
	pub async fn HealthCheck(&self) -> Result<bool, CommonError> {
		if !self.client.is_connected() {
			return Ok(false);
		}

		match self.GetStatus().await {
			Ok(_response) => Ok(true),
			Err(_) => Ok(false),
		}
	}
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Generates a unique request ID for Air operations.
///
/// Uses UUID v4 to generate a cryptographically random unique identifier.
/// This is used to correlate requests with responses and for tracing.
fn generate_request_id() -> String {
	Uuid::new_v4().simple().to_string()
}

/// Creates a new AirServiceProvider by attempting to connect to Air.
///
/// # Arguments
/// * `address` - The gRPC server address (e.g., "http://[::1]:50053")
///
/// # Returns
/// * `Ok(AirServiceProvider)` - Successfully connected provider
/// * `Err(CommonError)` - Connection failure
pub async fn CreateAirServiceProvider(address: &str) -> Result<AirServiceProvider, CommonError> {
	let client = Arc::new(AirClient::new(address).await?);
	Ok(AirServiceProvider::new(client))
}

/// Creates a new AirServiceProvider with graceful handling for unavailable Air.
///
/// If Air is not available, this creates an uninitialized client that will
/// return appropriate errors when operations are attempted.
///
/// # Arguments
/// * `address` - The gRPC server address (e.g., "http://[::1]:50053")
///
/// # Returns
/// An AirServiceProvider (may be uninitialized if Air is unavailable)
pub fn CreateAirServiceProviderOrUnavailable(address: &str) -> AirServiceProvider {
	let client = Arc::new(AirClient::new_or_unavailable(address));
	AirServiceProvider::new(client)
}
