// File: Mountain/Source/Air/AirClient.rs
// Role: gRPC client wrapper for Air daemon service
// Responsibilities:
//   - Manage gRPC connection to Air service
//   - Implement all Air service methods
//   - Translate tonic errors to CommonError
//   - Provide connection retry capabilities (optional)

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, time::Duration};

use Common::Error::CommonError::CommonError;
use log::{debug, error, info, warn};
use tonic::transport::{Channel, Endpoint};

// Import generated Air types from Air element
use Air::Vine::Generated::air::air_service_client;

// Re-export Air types for external use
pub use Air::Vine::Generated::air::{
    AuthenticationRequest, AuthenticationResponse,
    UpdateCheckRequest, UpdateCheckResponse,
    ApplyUpdateRequest, ApplyUpdateResponse,
    DownloadRequest, DownloadResponse,
    IndexRequest, IndexResponse,
    SearchRequest, SearchResponse,
    FileResult,
    StatusRequest, StatusResponse,
    MetricsRequest, MetricsResponse,
};

/// Default gRPC server address for the Air daemon.
///
/// Port Allocation:
/// - 50051: Mountain Vine server
/// - 50052: Cocoon Vine server (VS Code extension hosting)
/// - 50053: Air Vine server (Air daemon services - authentication, updates, and more)
pub const DEFAULT_AIR_SERVER_ADDRESS: &str = "[::1]:50053";

/// Air gRPC client wrapper that handles connection to the Air daemon service.
/// This provides a clean interface for Mountain to interact with Air's
/// capabilities including update management, authentication, file indexing,
/// and system monitoring.
#[derive(Clone)]
pub struct AirClient {
	// The underlying tonic gRPC client
	inner: Option<air_service_client::AirServiceClient<Channel>>,
	address: String,
}

impl AirClient {
	/// Creates a new AirClient and connects to the Air daemon service.
	///
	/// # Arguments
	/// * `address` - The gRPC server address (e.g., "http://[::1]:50053")
	///
	/// # Returns
	/// * `Ok(Self)` - Successfully connected client
	/// * `Err(CommonError)` - Connection failure with descriptive error
	pub async fn new(address: &str) -> Result<Self, CommonError> {
		info!("[AirClient] Attempting to connect to Air at: {}", address);

		// Attempt to connect with timeout
		let endpoint = Endpoint::from_shared(address.to_string())
			.map_err(|e| CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Invalid endpoint address: {}", e),
			})?;

		match tokio::time::timeout(Duration::from_secs(5), endpoint.connect()).await {
			Ok(Ok(channel)) => {
				info!("[AirClient] Successfully connected to Air at: {}", address);
				let client = air_service_client::AirServiceClient::new(channel);
				Ok(Self {
					inner: Some(client),
					address: address.to_string(),
				})
			},
			Ok(Err(e)) => {
				let description = format!("Failed to establish gRPC connection: {}", e);
				error!("[AirClient] Connection error: {}", description);
				Err(CommonError::ExternalServiceError {
					ServiceName: "Air".to_string(),
					Description: description,
				})
			},
			Err(_) => {
				let description = "Connection timed out after 5 seconds".to_string();
				warn!("[AirClient] Connection timeout: {}", description);
				Err(CommonError::ExternalServiceError {
					ServiceName: "Air".to_string(),
					Description: description,
				})
			},
		}
	}

	/// Creates a new AirClient without strictly requiring a connection.
	/// Returns an uninitialized client if Air is not available.
	///
	/// This is useful for graceful degradation when Air may not be running.
	pub fn new_or_unavailable(address: &str) -> Self {
		warn!("[AirClient] Air not available at: {}. Creating uninitialized client.", address);
		Self {
			inner: None,
			address: address.to_string(),
		}
	}

	/// Checks if the Air client is connected and ready to accept requests.
	pub fn is_connected(&self) -> bool {
		self.inner.is_some()
	}

	/// Gets the Air service address.
	pub fn address(&self) -> &str {
		&self.address
	}

	// =========================================================================
	// Update Operations
	// =========================================================================

	/// Checks for available updates for the application.
	///
	/// # Arguments
	/// * `request` - Update check parameters including current version and channel
	///
	/// # Returns
	/// Response containing available update information or error
	pub async fn CheckForUpdates(&self, request: UpdateCheckRequest) -> Result<UpdateCheckResponse, CommonError> {
		debug!("[AirClient] CheckForUpdates request_id={}", request.request_id);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.check_for_updates(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	/// Downloads a file from a specified URL.
	///
	/// # Arguments
	/// * `request` - Download parameters including URL and destination path
	///
	/// # Returns
	/// Response containing download result or error
	pub async fn DownloadFile(&self, request: DownloadRequest) -> Result<DownloadResponse, CommonError> {
		debug!("[AirClient] DownloadFile request_id={}, url={}", request.request_id, request.url);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.download_file(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	// =========================================================================
	// Authentication Operations
	// =========================================================================

	/// Authenticates a user with the specified credentials and provider.
	///
	/// # Arguments
	/// * `request` - Authentication request with username, password, and provider
	///
	/// # Returns
	/// Response containing authentication token or error
	pub async fn Authenticate(&self, request: AuthenticationRequest) -> Result<AuthenticationResponse, CommonError> {
		debug!("[AirClient] Authenticate request_id={}, provider={}", request.request_id, request.provider);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.authenticate(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	// =========================================================================
	// File Indexing Operations
	// =========================================================================

	/// Indexes files in the specified path for search functionality.
	///
	/// # Arguments
	/// * `request` - Index request with path, patterns, and depth
	///
	/// # Returns
	/// Response containing indexing statistics or error
	pub async fn IndexFiles(&self, request: IndexRequest) -> Result<IndexResponse, CommonError> {
		debug!(
			"[AirClient] IndexFiles request_id={}, path={}, patterns={:?}",
			request.request_id, request.path, request.patterns
		);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.index_files(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	/// Searches indexed files for the specified query.
	///
	/// # Arguments
	/// * `request` - Search request with query and path constraints
	///
	/// # Returns
	/// Response containing search results or error
	pub async fn SearchFiles(&self, request: SearchRequest) -> Result<SearchResponse, CommonError> {
		debug!(
			"[AirClient] SearchFiles request_id={}, query={}, path={}",
			request.request_id, request.query, request.path
		);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.search_files(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	// =========================================================================
	// Status and Monitoring Operations
	// =========================================================================

	/// Gets the current status of the Air daemon.
	///
	/// # Arguments
	/// * `request` - Status request (minimal structure)
	///
	/// # Returns
	/// Response containing system status metrics
	pub async fn GetStatus(&self, request: StatusRequest) -> Result<StatusResponse, CommonError> {
		debug!("[AirClient] GetStatus request_id={}", request.request_id);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.get_status(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	/// Gets detailed metrics from the Air daemon.
	///
	/// # Arguments
	/// * `request` - Metrics request specifying type of metrics to retrieve
	///
	/// # Returns
	/// Response containing requested metrics
	pub async fn GetMetrics(&self, request: MetricsRequest) -> Result<MetricsResponse, CommonError> {
		debug!("[AirClient] GetMetrics request_id={}, type={}", request.request_id, request.metric_type);

		self.ensure_connected()?;

		let response = self.inner.as_ref().unwrap()
			.get_metrics(request)
			.await
			.map_err(|e| Self::from_tonic_status(e))?
			.into_inner();

		Ok(response)
	}

	// =========================================================================
	// Connection Management
	// =========================================================================

	/// Attempts to reconnect to the Air daemon.
	///
	/// This is useful for recovering from transient network failures.
	/// Returns an error if reconnection fails.
	pub async fn reconnect(&mut self) -> Result<(), CommonError> {
		info!("[AirClient] Attempting to reconnect to Air at: {}", self.address);

		let endpoint = Endpoint::from_shared(self.address.clone())
			.map_err(|e| CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Invalid endpoint address: {}", e),
			})?;

		match tokio::time::timeout(Duration::from_secs(5), endpoint.connect()).await {
			Ok(Ok(channel)) => {
				let client = air_service_client::AirServiceClient::new(channel);
				self.inner = Some(client);
				info!("[AirClient] Successfully reconnected to Air");
				Ok(())
			},
			Ok(Err(e)) => {
				let description = format!("Failed to reconnect: {}", e);
				error!("[AirClient] Reconnection error: {}", description);
				Err(CommonError::ExternalServiceError {
					ServiceName: "Air".to_string(),
					Description: description,
				})
			},
			Err(_) => {
				let description = "Reconnection timed out after 5 seconds".to_string();
				warn!("[AirClient] Reconnection timeout: {}", description);
				Err(CommonError::ExternalServiceError {
					ServiceName: "Air".to_string(),
					Description: description,
				})
			},
		}
	}

	// =========================================================================
	// Helper Methods
	// =========================================================================

	/// Ensures the client is connected before making a request.
	fn ensure_connected(&self) -> Result<(), CommonError> {
		if self.inner.is_none() {
			error!("[AirClient] Attempted to call Air but no connection is established");
			Err(CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: "Air client is not connected. Air service may be unavailable.".to_string(),
			})
		} else {
			Ok(())
		}
	}

	/// Translates a tonic::Status error to CommonError.
	///
	/// This converts gRPC-level errors into our application's CommonError type
	/// for consistent error handling throughout Mountain.
	#[allow(dead_code)]
	fn from_tonic_status(status: tonic::Status) -> CommonError {
		use tonic::Code;

		match status.code() {
			Code::Unavailable | Code::Aborted | Code::Cancelled => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Service unavailable: {}", status.message()),
			},
			Code::Unauthenticated => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Authentication failed: {}", status.message()),
			},
			Code::PermissionDenied => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Permission denied: {}", status.message()),
			},
			Code::InvalidArgument => CommonError::InvalidArgument {
				ArgumentName: "request".to_string(),
				Reason: status.message().to_string(),
			},
			Code::NotFound => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("Resource not found: {}", status.message()),
			},
			Code::DeadlineExceeded => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: "Request timed out".to_string(),
			},
			_ => CommonError::ExternalServiceError {
				ServiceName: "Air".to_string(),
				Description: format!("gRPC error (code={:?}): {}", status.code(), status.message()),
			},
		}
	}
}
