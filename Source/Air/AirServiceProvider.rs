//! # AirServiceProvider
//!
//! High-level API surface for Air service methods.
//!
//! ## RESPONSIBILITIES
//!
//! - **Service Facade**: Provide convenient, high-level interface to Air daemon
//! - **Authentication**: Manage user authentication and credentials
//! - **Updates**: Check for and download application updates
//! - **File Indexing**: Query Air's file search and indexing capabilities
//! - **System Monitoring**: Retrieve system metrics and health data
//! - **Graceful Degradation**: Handle Air unavailability with fallbacks
//!
//! ## ARCHITECTURAL ROLE
//!
//! AirServiceProvider acts as a facade over the raw `AirClient`, providing:
//! - Simplified API for common operations
//! - Automatic error handling and translation
//! - Request ID generation for tracing
//! - Connection state management
//!
//! ```text
//! Application ──► AirServiceProvider ──► AirClient ──► gRPC ──► Air Daemon
//! ```
//!
//! ### Dependencies
//! - `AirClient`: Low-level gRPC client
//! - `uuid`: For generating request identifiers
//! - `CommonLibrary::Error::CommonError`: Error types
//!
//! ### Dependents
//! - `Binary::Service::VineStart`: Initializes Air service
//! - `MountainEnvironment`: Can delegate to Air when available
//!
//! ## IMPLEMENTATION
//!
//! This implementation provides a fully functional provider that wraps the
//! AirClient type with automatic request ID generation and error handling.
//!
//! ## ERROR HANDLING
//!
//! All operations return `Result<T, CommonError>` with:
//! - Translated gRPC errors to appropriate CommonError types
//! - Request IDs included in logs for tracing
//! - Graceful fallback to local operations when Air is unavailable
//!
//! ## PERFORMANCE
//!
//! - Request ID generation uses UUID v4 (cryptographically random)
//! - Thread-safe operations via `Arc<AirClient>`
//! - Non-blocking async operations via tokio
//!
//! ## VSCODE REFERENCE
//!
//! Patterns borrowed from VS Code:
//! - `vs/platform/update/common/updateService.ts` - Update management
//! - `vs/platform/authentication/common/authenticationService.ts` - Auth
//!   handling
//! - `vs/platform/filesystem/common/filesystem.ts` - File indexing
//!
//! ## MODULE CONTENTS
//!
//! - [`AirServiceProvider`]: Main provider struct
//! - [`generate_request_id`]: Helper function for UUID generation

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::Error::CommonError::CommonError;
use uuid::Uuid;

#[allow(unused_imports)]
use super::{
	AirClient::{
		AirClient,
		AirMetrics,
		AirStatus,
		DownloadStream,
		DownloadStreamChunk,
		ExtendedFileInfo,
		FileInfo,
		FileResult,
		IndexInfo,
		ResourceUsage,
		UpdateInfo,
	},
	AirClient::DEFAULT_AIR_SERVER_ADDRESS,
};
use crate::dev_log;

// ============================================================================
// AirServiceProvider - High-level API Implementation
// ============================================================================

/// AirServiceProvider provides a high-level, convenient interface to the Air
/// daemon service.
///
/// This provider wraps the AirClient and provides simplified methods with
/// automatic request ID generation and error handling. It acts as a facade
/// pattern, hiding the complexity of gRPC communication from the rest of the
/// Mountain application.
///
/// # Example
///
/// ```text
/// use Mountain::Air::AirServiceProvider::{AirServiceProvider, DEFAULT_AIR_SERVER_ADDRESS};
/// use CommonLibrary::Error::CommonError::CommonError;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), CommonError> {
/// let provider = AirServiceProvider::new(DEFAULT_AIR_SERVER_ADDRESS.to_string()).await?;
///
/// // Check for health
/// let is_healthy = provider.health_check().await?;
/// println!("Air service healthy: {}", is_healthy);
///
/// // Check for updates
/// if let Some(update) =
/// 	provider.check_for_updates("1.0.0".to_string(), "stable".to_string()).await?
/// {
/// 	println!("Update available: {}", update.version);
/// }
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AirServiceProvider {
	/// The underlying Air client wrapped in Arc for thread safety
	client:Arc<AirClient>,
}

impl AirServiceProvider {
	/// Creates a new AirServiceProvider and connects to the Air daemon.
	///
	/// # Arguments
	/// * `address` - The gRPC server address (defaults to `[::1]:50053`)
	///
	/// # Returns
	/// * `Ok(Self)` - Successfully created provider
	/// * `Err(CommonError)` - Connection failure
	///
	/// # Example
	///
	/// ```text
	/// use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// use CommonLibrary::Error::CommonError::CommonError;
	///
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// let provider = AirServiceProvider::new("http://[::1]:50053".to_string()).await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn new(address:String) -> Result<Self, CommonError> {
		dev_log!("grpc", "[AirServiceProvider] Creating AirServiceProvider at: {}", address);

		let client = AirClient::new(&address).await?;

		dev_log!("grpc", "[AirServiceProvider] AirServiceProvider created successfully");

		Ok(Self { client:Arc::new(client) })
	}

	/// Creates a new AirServiceProvider with the default address.
	///
	/// This is a convenience method that uses [`DEFAULT_AIR_SERVER_ADDRESS`].
	///
	/// # Returns
	/// * `Ok(Self)` - Successfully created provider
	/// * `Err(CommonError)` - Connection failure
	///
	/// # Example
	///
	/// ```text
	/// use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// use CommonLibrary::Error::CommonError::CommonError;
	///
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// let provider = AirServiceProvider::new_default().await?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn new_default() -> Result<Self, CommonError> { Self::new(DEFAULT_AIR_SERVER_ADDRESS.to_string()).await }

	/// Creates a new AirServiceProvider from an existing AirClient.
	///
	/// This is useful when you need to share a client or have special
	/// connection requirements.
	///
	/// # Arguments
	/// * `client` - The AirClient to wrap
	///
	/// # Returns
	/// * `Self` - The new provider
	pub fn from_client(client:Arc<AirClient>) -> Self {
		dev_log!("grpc", "[AirServiceProvider] Creating AirServiceProvider from existing client");
		Self { client }
	}

	/// Gets a reference to the underlying AirClient.
	///
	/// This provides access to the low-level client when needed.
	///
	/// # Returns
	/// Reference to the AirClient
	pub fn client(&self) -> &Arc<AirClient> { &self.client }

	/// Checks if the provider is connected to Air.
	///
	/// # Returns
	/// * `true` - Connected
	/// * `false` - Not connected
	pub fn is_connected(&self) -> bool { self.client.is_connected() }

	/// Gets the address of the Air daemon.
	///
	/// # Returns
	/// The address string
	pub fn address(&self) -> &str { self.client.address() }

	// =========================================================================
	// Authentication Operations
	// =========================================================================

	/// Authenticates a user with the Air daemon.
	///
	/// This method handles request ID generation and provides a simplified
	/// interface for authentication.
	///
	/// # Arguments
	/// * `username` - User's username
	/// * `password` - User's password
	/// * `provider` - Authentication provider (e.g., "github", "gitlab",
	///   "microsoft")
	///
	/// # Returns
	/// * `Ok(token)` - Authentication token if successful
	/// * `Err(CommonError)` - Authentication error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// let token = provider
	/// 	.authenticate("user@example.com".to_string(), "password".to_string(), "github".to_string())
	/// 	.await?;
	/// println!("Auth token: {}", token);
	/// # Ok(())
	/// # }
	/// ```
	pub async fn authenticate(&self, username:String, password:String, provider:String) -> Result<String, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] authenticate (request_id: {})", request_id);

		self.client.authenticate(request_id, username, password, provider).await
	}

	// =========================================================================
	// Update Operations
	// =========================================================================

	/// Checks for available updates.
	///
	/// Returns None if no update is available, Some with update info otherwise.
	///
	/// # Arguments
	/// * `current_version` - Current application version
	/// * `channel` - Update channel (e.g., "stable", "beta", "nightly")
	///
	/// # Returns
	/// * `Ok(Some(update))` - Update available with information
	/// * `Ok(None)` - No update available
	/// * `Err(CommonError)` - Check error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// if let Some(update) =
	/// 	provider.check_for_updates("1.0.0".to_string(), "stable".to_string()).await?
	/// {
	/// 	println!("Update available: version {}", update.version);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub async fn check_for_updates(
		&self,
		current_version:String,
		channel:String,
	) -> Result<Option<UpdateInfo>, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] check_for_updates (request_id: {})", request_id);

		let info = self.client.check_for_updates(request_id, current_version, channel).await?;

		if info.update_available { Ok(Some(info)) } else { Ok(None) }
	}

	/// Downloads an update package.
	///
	/// # Arguments
	/// * `url` - URL of the update package
	/// * `destination_path` - Local path to save the downloaded file
	/// * `checksum` - Optional SHA256 checksum for verification
	///
	/// # Returns
	/// * `Ok(file_info)` - Downloaded file information
	/// * `Err(CommonError)` - Download error
	pub async fn download_update(
		&self,
		url:String,
		destination_path:String,
		checksum:String,
	) -> Result<FileInfo, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] download_update (request_id: {})", request_id);

		self.client
			.download_update(request_id, url, destination_path, checksum, HashMap::new())
			.await
	}

	/// Applies an update package.
	///
	/// # Arguments
	/// * `version` - Version of the update
	/// * `update_path` - Path to the update package
	///
	/// # Returns
	/// * `Ok(())` - Update applied successfully
	/// * `Err(CommonError)` - Application error
	pub async fn apply_update(&self, version:String, update_path:String) -> Result<(), CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] apply_update (request_id: {})", request_id);

		self.client.apply_update(request_id, version, update_path).await
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
	///
	/// # Returns
	/// * `Ok(file_info)` - Downloaded file information
	/// * `Err(CommonError)` - Download error
	pub async fn download_file(
		&self,
		url:String,
		destination_path:String,
		checksum:String,
	) -> Result<FileInfo, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] download_file (request_id: {})", request_id);

		self.client
			.download_file(request_id, url, destination_path, checksum, HashMap::new())
			.await
	}

	/// Downloads a file as a stream.
	///
	/// This method initiates a streaming download from the given URL, returning
	/// a stream of chunks that can be processed incrementally without loading
	/// the entire file into memory.
	///
	/// # Arguments
	/// * `url` - URL of the file to download
	/// * `headers` - Optional HTTP headers
	///
	/// # Returns
	/// * `Ok(stream)` - Stream that yields download chunks
	/// * `Err(CommonError)` - Download error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// let mut stream = provider
	/// 	.download_stream(
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
		url:String,
		headers:HashMap<String, String>,
	) -> Result<DownloadStream, CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] download_stream (request_id: {}, url: {})",
			request_id,
			url
		);

		self.client.download_stream(request_id, url, headers).await
	}

	// =========================================================================
	// File Indexing Operations
	// =========================================================================

	/// Indexes files in a directory.
	///
	/// # Arguments
	/// * `path` - Path to the directory to index
	/// * `patterns` - File patterns to include (e.g., ["*.rs", "*.ts"])
	/// * `exclude_patterns` - File patterns to exclude (e.g.,
	///   ["node_modules/*"])
	/// * `max_depth` - Maximum depth for recursion (0 for unlimited)
	///
	/// # Returns
	/// * `Ok(index_info)` - Index information with file count and total size
	/// * `Err(CommonError)` - Indexing error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// let info = provider
	/// 	.index_files(
	/// 		"/path/to/project".to_string(),
	/// 		vec!["*.rs".to_string(), "*.ts".to_string()],
	/// 		vec!["node_modules/*".to_string()],
	/// 		10,
	/// 	)
	/// 	.await?;
	/// println!("Indexed {} files ({} bytes)", info.files_indexed, info.total_size);
	/// # Ok(())
	/// # }
	/// ```
	pub async fn index_files(
		&self,
		path:String,
		patterns:Vec<String>,
		exclude_patterns:Vec<String>,
		max_depth:u32,
	) -> Result<IndexInfo, CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] index_files (request_id: {}, path: {})",
			request_id,
			path
		);

		self.client
			.index_files(request_id, path, patterns, exclude_patterns, max_depth)
			.await
	}

	/// Searches for files matching a query.
	///
	/// # Arguments
	/// * `query` - Search query string
	/// * `path` - Path to search in (empty for entire workspace)
	/// * `max_results` - Maximum number of results to return (0 for unlimited)
	///
	/// # Returns
	/// * `Ok(results)` - Vector of file search results
	/// * `Err(CommonError)` - Search error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// let results = provider
	/// 	.search_files("fn main".to_string(), "/path/to/project".to_string(), 50)
	/// 	.await?;
	/// for result in results {
	/// 	println!("Found: {} at line {}", result.path, result.line_number);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub async fn search_files(
		&self,
		query:String,
		path:String,
		max_results:u32,
	) -> Result<Vec<FileResult>, CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] search_files (request_id: {}, query: {})",
			request_id,
			query
		);

		self.client.search_files(request_id, query, path, max_results).await
	}

	/// Gets file information.
	///
	/// # Arguments
	/// * `path` - Path to the file
	///
	/// # Returns
	/// * `Ok(file_info)` - Extended file information
	/// * `Err(CommonError)` - Request error
	pub async fn get_file_info(&self, path:String) -> Result<ExtendedFileInfo, CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] get_file_info (request_id: {}, path: {})",
			request_id,
			path
		);

		self.client.get_file_info(request_id, path).await
	}

	// =========================================================================
	// Status and Monitoring Operations
	// =========================================================================

	/// Gets the status of the Air daemon.
	///
	/// # Returns
	/// * `Ok(status)` - Air daemon status information
	/// * `Err(CommonError)` - Request error
	pub async fn get_status(&self) -> Result<AirStatus, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] get_status (request_id: {})", request_id);

		self.client.get_status(request_id).await
	}

	/// Performs a health check on the Air daemon.
	///
	/// # Returns
	/// * `Ok(healthy)` - Health status (true if healthy)
	/// * `Err(CommonError)` - Check error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// if provider.health_check().await? {
	/// 	println!("Air service is healthy");
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub async fn health_check(&self) -> Result<bool, CommonError> {
		dev_log!("grpc", "[AirServiceProvider] health_check");
		self.client.health_check().await
	}

	/// Gets metrics from the Air daemon.
	///
	/// # Arguments
	/// * `metric_type` - Optional type of metrics (e.g., "performance",
	///   "resources")
	///
	/// # Returns
	/// * `Ok(metrics)` - Metrics data
	/// * `Err(CommonError)` - Request error
	pub async fn get_metrics(&self, metric_type:Option<String>) -> Result<AirMetrics, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] get_metrics (request_id: {})", request_id);

		self.client.get_metrics(request_id, metric_type).await
	}

	// =========================================================================
	// Resource Management Operations
	// =========================================================================

	/// Gets resource usage information.
	///
	/// # Returns
	/// * `Ok(usage)` - Resource usage data
	/// * `Err(CommonError)` - Request error
	pub async fn get_resource_usage(&self) -> Result<ResourceUsage, CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] get_resource_usage (request_id: {})", request_id);

		self.client.get_resource_usage(request_id).await
	}

	/// Sets resource limits.
	///
	/// # Arguments
	/// * `memory_limit_mb` - Memory limit in MB
	/// * `cpu_limit_percent` - CPU limit as percentage (0-100)
	/// * `disk_limit_mb` - Disk limit in MB
	///
	/// # Returns
	/// * `Ok(())` - Limits set successfully
	/// * `Err(CommonError)` - Set error
	pub async fn set_resource_limits(
		&self,
		memory_limit_mb:u32,
		cpu_limit_percent:u32,
		disk_limit_mb:u32,
	) -> Result<(), CommonError> {
		let request_id = generate_request_id();
		dev_log!("grpc", "[AirServiceProvider] set_resource_limits (request_id: {})", request_id);

		self.client
			.set_resource_limits(request_id, memory_limit_mb, cpu_limit_percent, disk_limit_mb)
			.await
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
	/// * `Ok(config)` - Configuration data as key-value pairs
	/// * `Err(CommonError)` - Request error
	///
	/// # Example
	///
	/// ```text
	/// # use Mountain::Air::AirServiceProvider::AirServiceProvider;
	/// # use CommonLibrary::Error::CommonError::CommonError;
	/// # #[tokio::main]
	/// # async fn main() -> Result<(), CommonError> {
	/// # let provider = AirServiceProvider::new_default().await?;
	/// let config = provider.get_configuration("grpc".to_string()).await?;
	/// for (key, value) in config {
	/// 	println!("{} = {}", key, value);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub async fn get_configuration(&self, section:String) -> Result<HashMap<String, String>, CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] get_configuration (request_id: {}, section: {})",
			request_id,
			section
		);

		self.client.get_configuration(request_id, section).await
	}

	/// Updates configuration.
	///
	/// # Arguments
	/// * `section` - Configuration section
	/// * `updates` - Configuration updates as key-value pairs
	///
	/// # Returns
	/// * `Ok(())` - Configuration updated successfully
	/// * `Err(CommonError)` - Update error
	pub async fn update_configuration(
		&self,
		section:String,
		updates:HashMap<String, String>,
	) -> Result<(), CommonError> {
		let request_id = generate_request_id();
		dev_log!(
			"grpc",
			"[AirServiceProvider] update_configuration (request_id: {}, section: {})",
			request_id,
			section
		);

		self.client.update_configuration(request_id, section, updates).await
	}
}

// ============================================================================
// Helper Function - Request ID Generation
// ============================================================================

/// Generates a unique request ID for Air operations.
///
/// Uses UUID v4 to generate a cryptographically random unique identifier.
/// This is used to correlate requests with responses and for tracing.
///
/// # Returns
/// A UUID string in simple format (without dashes)
///
/// # Example
///
/// ```text
/// use Mountain::Air::AirServiceProvider::generate_request_id;
///
/// let id = generate_request_id();
/// println!("Request ID: {}", id);
/// // Output example: Request ID: a1b2c3d4e5f67890...
/// ```
pub fn generate_request_id() -> String { Uuid::new_v4().simple().to_string() }

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_generate_request_id() {
		let id1 = generate_request_id();
		let id2 = generate_request_id();

		// IDs should be unique
		assert_ne!(id1, id2);

		// IDs should be valid UUIDs (simple format = 32 chars)
		assert_eq!(id1.len(), 32);
		assert_eq!(id2.len(), 32);

		// IDs should be hex characters
		assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
		assert!(id2.chars().all(|c| c.is_ascii_hexdigit()));
	}

	#[test]
	fn test_default_address() {
		assert_eq!(DEFAULT_AIR_SERVER_ADDRESS, "[::1]:50053");
	}
}
