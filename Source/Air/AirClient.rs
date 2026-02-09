//! # AirClient
//!
//! gRPC client wrapper for the Air daemon service, providing Mountain with
//! access to cloud-based backend services including updates, authentication,
//! file indexing, and system monitoring.
//!
//! ## RESPONSIBILITIES
//!
//! - **Connection Management**: Manage gRPC connection lifecycle to Air service
//! - **Service Methods**: Implement all Air service RPC methods
//! - **Error Translation**: Convert tonic/transport errors to CommonError
//! - **Connection Retry**: (Optional) Provide automatic retry with backoff
//! - **Health Checking**: Monitor Air service availability
//!
//! ## ARCHITECTURAL ROLE
//!
//! AirClient serves as the primary interface between Mountain and the Air
//! backend service:
//!
//! ```
//! Mountain (Frontend) ──► AirClient ──► gRPC ──► Air Daemon (Backend)
//! ```
//!
//! ### Position in Mountain
//! - Communication module for Air integration
//! - Part of the service management layer
//! - Features-gated behind `AirIntegration` feature flag
//!
//! ## IMPLEMENTATION
//!
//! This implementation uses the generated gRPC client from the Air library:
//! - `AirLibrary::Vine::Generated::air_service_client::AirServiceClient`
//!
//! ## CONFIGURATION
//!
//! - **Default Address**: `[::1]:50053` (configurable via constructor)
//! - **Transport**: gRPC over TCP/IP with optional TLS
//! - **Connection Pooling**: (TODO) Implement for multiple concurrent requests
//!
//! ## ERROR HANDLING
//!
//! All methods return `Result<T, CommonError>` with appropriate error types:
//! - `IPCError`: gRPC communication failures
//! - `SerializationError`: Message encoding/decoding failures
//! - `Unknown`: Uncategorized errors
//!
//! ## THREAD SAFETY
//!
//! - `AirClient` is `Clone`able and can be shared across threads via
//!   `Arc<AirClient>`
//! - The underlying tonic client is thread-safe
//! - All public methods are safe to call from multiple threads
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Connection establishment is lazy (deferred until first use)
//! - (TODO) Implement connection pooling for high-throughput scenarios
//! - (TODO) Add request caching for frequently accessed data
//! - (TODO) Implement request timeout configuration
//!
//! ## TODO
//!
//! High Priority:
//! - [ ] Add connection retry with exponential backoff
//! - [ ] Implement proper connection pooling
//!
//! Medium Priority:
//! - [ ] Add request/response logging for debugging
//! - [ ] Implement connection health monitoring
//! - [ ] Add metrics collection for RPC calls
//!
//! ## MODULE CONTENTS
//!
//! - [`AirClient`]: Main client struct
//! - [`DEFAULT_AIR_SERVER_ADDRESS`]: Default gRPC server address constant

use std::collections::HashMap;
use std::sync::Arc;

use CommonLibrary::Error::CommonError::CommonError;
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use log::{debug, error, info, warn};
use tonic::transport::Channel;
use tonic::Request;

/// Default gRPC server address for the Air daemon.
///
/// Port Allocation:
/// - 50051: Mountain Vine server
/// - 50052: Cocoon Vine server (VS Code extension hosting)
/// - 50053: Air Vine server (Air daemon services - authentication, updates, and
///   more)
pub const DEFAULT_AIR_SERVER_ADDRESS: &str = "[::1]:50053";

/// Air gRPC client wrapper that handles connection to the Air daemon service.
/// This provides a clean interface for Mountain to interact with Air's
/// capabilities including update management, authentication, file indexing,
/// and system monitoring.
#[derive(Debug, Clone)]
pub struct AirClient {
    #[cfg(feature = "AirIntegration")]
    /// The underlying tonic gRPC client
    client: Arc<AirServiceClient<Channel>>,
    /// Address of the Air daemon
    address: String,
}

impl AirClient {
    /// Creates a new AirClient and connects to the Air daemon service.
    ///
    /// # Arguments
    /// * `address` - The gRPC server address (e.g., "http://[::1]:50053")
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully created client
    /// * `Err(CommonError)` - Connection failure with descriptive error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use Mountain::Air::AirClient::{AirClient, DEFAULT_AIR_SERVER_ADDRESS};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AirClient::new(DEFAULT_AIR_SERVER_ADDRESS).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(address: &str) -> Result<Self, CommonError> {
        info!("[AirClient] Connecting to Air daemon at: {}", address);

        #[cfg(feature = "AirIntegration")]
        {
            let endpoint = address.parse::<tonic::transport::Endpoint>().map_err(|e| {
                error!("[AirClient] Failed to parse address '{}': {}", address, e);
                CommonError::IPCError {
                    Description: format!("Invalid address '{}': {}", address, e),
                }
            })?;

            let channel = endpoint.connect().await.map_err(|e| {
                error!("[AirClient] Failed to connect to Air daemon: {}", e);
                CommonError::IPCError {
                    Description: format!("Connection failed: {}", e),
                }
            })?;

            info!("[AirClient] Successfully connected to Air daemon at: {}", address);

            Ok(Self {
                client: Arc::new(AirServiceClient::new(channel)),
                address: address.to_string(),
            })
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            error!("[AirClient] AirIntegration feature is not enabled");
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
            true // Assuming connection is established if client exists
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
    pub fn address(&self) -> &str {
        &self.address
    }

    // =========================================================================
    // Authentication Operations
    // =========================================================================

    /// Authenticates a user with the Air daemon.
    ///
    /// # Arguments
    /// * `username` - User's username
    /// * `password` - User's password
    /// * `provider` - Authentication provider (e.g., "github", "gitlab", "microsoft")
    ///
    /// # Returns
    /// * `Ok(token)` - Authentication token if successful
    /// * `Err(CommonError)` - Authentication failure
    pub async fn authenticate(
        &self,
        request_id: String,
        username: String,
        password: String,
        provider: String,
    ) -> Result<String, CommonError> {
        debug!("[AirClient] Authenticating user '{}' with provider '{}'", username, provider);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::AuthenticationRequest;

            let request = AuthenticationRequest {
                request_id,
                username,
                password,
                provider,
            };

            let mut client = Arc::clone(&self.client);

            match client.authenticate(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!("[AirClient] Authentication successful for user '{}'", username);
                        Ok(response.token)
                    } else {
                        error!(
                            "[AirClient] Authentication failed for user '{}': {}",
                            username, response.error
                        );
                        Err(CommonError::AccessDenied {
                            Reason: response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Authentication RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Authentication RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        current_version: String,
        channel: String,
    ) -> Result<UpdateInfo, CommonError> {
        debug!(
            "[AirClient] Checking for updates for version '{}'",
            current_version
        );

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::UpdateCheckRequest;

            let request = UpdateCheckRequest {
                request_id,
                current_version,
                channel,
            };

            let mut client = Arc::clone(&self.client);

            match client.check_for_updates(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!(
                        "[AirClient] Update check completed. Update available: {}",
                        response.update_available
                    );
                    Ok(UpdateInfo {
                        update_available: response.update_available,
                        version: response.version,
                        download_url: response.download_url,
                        release_notes: response.release_notes,
                    })
                }
                Err(e) => {
                    error!("[AirClient] Check for updates RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Check for updates RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        url: String,
        destination_path: String,
        checksum: String,
        headers: HashMap<String, String>,
    ) -> Result<FileInfo, CommonError> {
        debug!("[AirClient] Downloading update from: {}", url);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::DownloadRequest;

            let request = DownloadRequest {
                request_id,
                url,
                destination_path,
                checksum,
                headers,
            };

            let mut client = Arc::clone(&self.client);

            match client.download_update(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!(
                            "[AirClient] Update downloaded successfully to: {}",
                            response.file_path
                        );
                        Ok(FileInfo {
                            file_path: response.file_path,
                            file_size: response.file_size,
                            checksum: response.checksum,
                        })
                    } else {
                        error!("[AirClient] Update download failed: {}", response.error);
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Download update RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Download update RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
    pub async fn apply_update(
        &self,
        request_id: String,
        version: String,
        update_path: String,
    ) -> Result<(), CommonError> {
        debug!("[AirClient] Applying update version: {}", version);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::ApplyUpdateRequest;

            let request = ApplyUpdateRequest {
                request_id,
                version,
                update_path,
            };

            let mut client = Arc::clone(&self.client);

            match client.apply_update(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!("[AirClient] Update applied successfully");
                        Ok(())
                    } else {
                        error!("[AirClient] Update application failed: {}", response.error);
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Apply update RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Apply update RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        url: String,
        destination_path: String,
        checksum: String,
        headers: HashMap<String, String>,
    ) -> Result<FileInfo, CommonError> {
        debug!("[AirClient] Downloading file from: {}", url);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::DownloadRequest;

            let request = DownloadRequest {
                request_id,
                url,
                destination_path,
                checksum,
                headers,
            };

            let mut client = Arc::clone(&self.client);

            match client.download_file(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!(
                            "[AirClient] File downloaded successfully to: {}",
                            response.file_path
                        );
                        Ok(FileInfo {
                            file_path: response.file_path,
                            file_size: response.file_size,
                            checksum: response.checksum,
                        })
                    } else {
                        error!("[AirClient] File download failed: {}", response.error);
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Download file RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Download file RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
    /// ```rust,no_run
    /// use Mountain::Air::AirClient::AirClient;
    /// use CommonLibrary::Error::CommonError::CommonError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), CommonError> {
    /// # let client = AirClient::new("http://[::1]:50053").await?;
    /// let mut stream = client.download_stream(
    ///     "req-123".to_string(),
    ///     "https://example.com/large-file.zip".to_string(),
    ///     std::collections::HashMap::new()
    /// ).await?;
    ///
    /// let mut buffer = Vec::new();
    /// while let Some(chunk) = stream.next().await {
    ///     let chunk = chunk?;
    ///     buffer.extend_from_slice(&chunk.data);
    ///     println!("Downloaded: {} / {} bytes", chunk.downloaded, chunk.total_size);
    ///     if chunk.completed {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_stream(
        &self,
        request_id: String,
        url: String,
        headers: HashMap<String, String>,
    ) -> Result<DownloadStream, CommonError> {
        debug!("[AirClient] Starting stream download from: {}", url);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::DownloadStreamRequest;

            let request = DownloadStreamRequest {
                request_id,
                url,
                headers,
            };

            let mut client = Arc::clone(&self.client);

            match client.download_stream(Request::new(request)).await {
                Ok(response) => {
                    info!("[AirClient] Stream download initiated successfully");
                    Ok(DownloadStream::new(response.into_inner()))
                }
                Err(e) => {
                    error!("[AirClient] Download stream RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Download stream RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        path: String,
        patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        max_depth: u32,
    ) -> Result<IndexInfo, CommonError> {
        debug!("[AirClient] Indexing files in: {}", path);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::IndexRequest;

            let request = IndexRequest {
                request_id,
                path,
                patterns,
                exclude_patterns,
                max_depth,
            };

            let mut client = Arc::clone(&self.client);

            match client.index_files(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!(
                            "[AirClient] Files indexed: {} (total size: {} bytes)",
                            response.files_indexed, response.total_size
                        );
                        Ok(IndexInfo {
                            files_indexed: response.files_indexed,
                            total_size: response.total_size,
                        })
                    } else {
                        error!("[AirClient] File indexing failed: {}", response.error);
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Index files RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Index files RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        query: String,
        path: String,
        max_results: u32,
    ) -> Result<Vec<FileResult>, CommonError> {
        debug!("[AirClient] Searching for files with query: '{}' in: {}", query, path);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::SearchRequest;

            let request = SearchRequest {
                request_id,
                query,
                path,
                max_results,
            };

            let mut client = Arc::clone(&self.client);

            match client.search_files(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!(
                        "[AirClient] Search completed. Found {} results",
                        response.total_results
                    );
                    let results = response
                        .results
                        .into_iter()
                        .map(|r| FileResult {
                            path: r.path,
                            size: r.size,
                            match_preview: r.match_preview,
                            line_number: r.line_number,
                        })
                        .collect();
                    Ok(results)
                }
                Err(e) => {
                    error!("[AirClient] Search files RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Search files RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
    pub async fn get_file_info(
        &self,
        request_id: String,
        path: String,
    ) -> Result<ExtendedFileInfo, CommonError> {
        debug!("[AirClient] Getting file info for: {}", path);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::FileInfoRequest;

            let request = FileInfoRequest {
                request_id,
                path,
            };

            let mut client = Arc::clone(&self.client);

            match client.get_file_info(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!(
                        "[AirClient] File info retrieved for: {} (exists: {})",
                        path, response.exists
                    );
                    Ok(ExtendedFileInfo {
                        exists: response.exists,
                        size: response.size,
                        mime_type: response.mime_type,
                        checksum: response.checksum,
                        modified_time: response.modified_time,
                    })
                }
                Err(e) => {
                    error!("[AirClient] Get file info RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Get file info RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
    pub async fn get_status(&self, request_id: String) -> Result<AirStatus, CommonError> {
        debug!("[AirClient] Getting Air daemon status");

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::StatusRequest;

            let request = StatusRequest { request_id };

            let mut client = Arc::clone(&self.client);

            match client.get_status(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!("[AirClient] Status retrieved. Active requests: {}", response.active_requests);
                    Ok(AirStatus {
                        version: response.version,
                        uptime_seconds: response.uptime_seconds,
                        total_requests: response.total_requests,
                        successful_requests: response.successful_requests,
                        failed_requests: response.failed_requests,
                        average_response_time: response.average_response_time,
                        memory_usage_mb: response.memory_usage_mb,
                        cpu_usage_percent: response.cpu_usage_percent,
                        active_requests: response.active_requests,
                    })
                }
                Err(e) => {
                    error!("[AirClient] Get status RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Get status RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
        }
    }

    /// Performs a health check on the Air daemon.
    ///
    /// # Returns
    /// * `Ok(healthy)` - Health status
    /// * `Err(CommonError)` - Check failure
    pub async fn health_check(&self) -> Result<bool, CommonError> {
        debug!("[AirClient] Performing health check");

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::HealthCheckRequest;

            let request = HealthCheckRequest {};

            let mut client = Arc::clone(&self.client);

            match client.health_check(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    debug!("[AirClient] Health check result: {}", response.healthy);
                    Ok(response.healthy)
                }
                Err(e) => {
                    error!("[AirClient] Health check RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Health check RPC error: {}", e),
                    })
                }
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
    /// * `metric_type` - Type of metrics (e.g., "performance", "resources", "requests")
    ///
    /// # Returns
    /// * `Ok(metrics)` - Metrics data
    /// * `Err(CommonError)` - Request failure
    pub async fn get_metrics(
        &self,
        request_id: String,
        metric_type: Option<String>,
    ) -> Result<AirMetrics, CommonError> {
        debug!(
            "[AirClient] Getting metrics (type: {:?})",
            metric_type.as_deref()
        );

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::MetricsRequest;

            let request = MetricsRequest {
                request_id,
                metric_type: metric_type.unwrap_or_default(),
            };

            let mut client = Arc::clone(&self.client);

            match client.get_metrics(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!("[AirClient] Metrics retrieved");
                    // Parse metrics from the string map - this is a simplified implementation
                    let metrics = AirMetrics {
                        memory_usage_mb: response.metrics.get("memory_usage_mb")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0),
                        cpu_usage_percent: response.metrics.get("cpu_usage_percent")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0),
                        network_usage_mbps: response.metrics.get("network_usage_mbps")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0),
                        disk_usage_mb: response.metrics.get("disk_usage_mb")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0),
                        average_response_time: response.metrics.get("average_response_time")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0),
                    };
                    Ok(metrics)
                }
                Err(e) => {
                    error!("[AirClient] Get metrics RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Get metrics RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
    pub async fn get_resource_usage(
        &self,
        request_id: String,
    ) -> Result<ResourceUsage, CommonError> {
        debug!("[AirClient] Getting resource usage");

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::ResourceUsageRequest;

            let request = ResourceUsageRequest { request_id };

            let mut client = Arc::clone(&self.client);

            match client.get_resource_usage(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!("[AirClient] Resource usage retrieved");
                    Ok(ResourceUsage {
                        memory_usage_mb: response.memory_usage_mb,
                        cpu_usage_percent: response.cpu_usage_percent,
                        disk_usage_mb: response.disk_usage_mb,
                        network_usage_mbps: response.network_usage_mbps,
                        thread_count: 0, // Not provided in ResourceUsageResponse
                        open_file_handles: 0, // Not provided in ResourceUsageResponse
                    })
                }
                Err(e) => {
                    error!("[AirClient] Get resource usage RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Get resource usage RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        memory_limit_mb: u32,
        cpu_limit_percent: u32,
        disk_limit_mb: u32,
    ) -> Result<(), CommonError> {
        debug!(
            "[AirClient] Setting resource limits: memory={}MB, cpu={}%, disk={}MB",
            memory_limit_mb, cpu_limit_percent, disk_limit_mb
        );

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::ResourceLimitsRequest;

            let request = ResourceLimitsRequest {
                request_id,
                memory_limit_mb,
                cpu_limit_percent,
                disk_limit_mb,
            };

            let mut client = Arc::clone(&self.client);

            match client.set_resource_limits(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!("[AirClient] Resource limits set successfully");
                        Ok(())
                    } else {
                        error!("[AirClient] Failed to set resource limits: {}", response.error);
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Set resource limits RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Set resource limits RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
        }
    }

    // =========================================================================
    // Configuration Management Operations
    // =========================================================================

    /// Gets configuration.
    ///
    /// # Arguments
    /// * `section` - Configuration section (e.g., "grpc", "authentication", "updates")
    ///
    /// # Returns
    /// * `Ok(config)` - Configuration data
    /// * `Err(CommonError)` - Request failure
    pub async fn get_configuration(
        &self,
        request_id: String,
        section: String,
    ) -> Result<HashMap<String, String>, CommonError> {
        debug!("[AirClient] Getting configuration for section: {}", section);

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::ConfigurationRequest;

            let request = ConfigurationRequest {
                request_id,
                section,
            };

            let mut client = Arc::clone(&self.client);

            match client.get_configuration(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    info!(
                        "[AirClient] Configuration retrieved for section: {} ({} keys)",
                        section,
                        response.configuration.len()
                    );
                    Ok(response.configuration)
                }
                Err(e) => {
                    error!("[AirClient] Get configuration RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Get configuration RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
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
        request_id: String,
        section: String,
        updates: HashMap<String, String>,
    ) -> Result<(), CommonError> {
        debug!(
            "[AirClient] Updating configuration for section: {} ({} keys)",
            section,
            updates.len()
        );

        #[cfg(feature = "AirIntegration")]
        {
            use AirLibrary::Vine::Generated::air::UpdateConfigurationRequest;

            let request = UpdateConfigurationRequest {
                request_id,
                section,
                updates,
            };

            let mut client = Arc::clone(&self.client);

            match client.update_configuration(Request::new(request)).await {
                Ok(response) => {
                    let response = response.into_inner();
                    if response.success {
                        info!(
                            "[AirClient] Configuration updated successfully for section: {}",
                            section
                        );
                        Ok(())
                    } else {
                        error!(
                            "[AirClient] Failed to update configuration: {}",
                            response.error
                        );
                        Err(CommonError::IPCError { Description: 
 response.error,
                        })
                    }
                }
                Err(e) => {
                    error!("[AirClient] Update configuration RPC error: {}", e);
                    Err(CommonError::IPCError { Description: 
 format!("Update configuration RPC error: {}", e),
                    })
                }
            }
        }

        #[cfg(not(feature = "AirIntegration"))]
        {
            Err(CommonError::FeatureNotAvailable {
                FeatureName: "AirIntegration".to_string(),
            })
        }
    }
}

// ============================================================================
// Response Types
// ============================================================================

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub update_available: bool,
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
}

/// Information about a downloaded file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file_path: String,
    pub file_size: u64,
    pub checksum: String,
}

/// Information about file indexing.
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub files_indexed: u32,
    pub total_size: u64,
}

/// Result of a file search.
#[derive(Debug, Clone)]
pub struct FileResult {
    pub path: String,
    pub size: u64,
    pub match_preview: String,
    pub line_number: u32,
}

/// Extended file information.
#[derive(Debug, Clone)]
pub struct ExtendedFileInfo {
    pub exists: bool,
    pub size: u64,
    pub mime_type: String,
    pub checksum: String,
    pub modified_time: u64,
}

/// Status of the Air daemon.
#[derive(Debug, Clone)]
pub struct AirStatus {
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

/// Metrics from the Air daemon.
#[derive(Debug, Clone)]
pub struct AirMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub network_usage_mbps: f64,
    pub disk_usage_mb: f64,
    pub average_response_time: f64,
}

/// Resource usage information.
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_usage_mbps: f64,
    pub thread_count: u32,
    pub open_file_handles: u32,
}

/// Chunk of data from a streaming download.
///
/// Each chunk represents a portion of the downloaded file with metadata
/// about the download progress.
#[derive(Debug, Clone)]
pub struct DownloadStreamChunk {
    /// Binary data chunk
    pub data: Vec<u8>,
    /// Total file size in bytes (0 if unknown)
    pub total_size: u64,
    /// Number of bytes downloaded so far
    pub downloaded: u64,
    /// Whether this is the final chunk
    pub completed: bool,
    /// Error message if download failed
    pub error: String,
}

/// Wrapper for an asynchronous download stream.
///
/// This type wraps the tonic streaming API to provide a convenient
/// interface for iterating over download chunks.
///
/// # Example
///
/// ```rust,no_run
/// use Mountain::Air::AirClient::DownloadStream;
/// use CommonLibrary::Error::CommonError::CommonError;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), CommonError> {
/// # let mut stream = DownloadStream::new(/* tonic stream */);
/// let mut buffer = Vec::new();
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     buffer.extend_from_slice(&chunk.data);
///     if chunk.completed {
///         break;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct DownloadStream {
    inner: tonic::codec::Streaming<
        AirLibrary::Vine::Generated::air::DownloadStreamResponse,
    >,
}

impl DownloadStream {
    /// Creates a new DownloadStream from a tonic streaming response.
    pub fn new(
        stream: tonic::codec::Streaming<
            AirLibrary::Vine::Generated::air::DownloadStreamResponse,
        >,
    ) -> Self {
        Self { inner: stream }
    }

    /// Returns the next chunk from the stream.
    ///
    /// Returns `None` when the stream ends.
    pub async fn next(&mut self) -> Option<Result<DownloadStreamChunk, CommonError>> {
        match self.inner.next().await {
            Some(Ok(response)) => {
                Some(Ok(DownloadStreamChunk {
                    data: response.chunk,
                    total_size: response.total_size,
                    downloaded: response.downloaded,
                    completed: response.completed,
                    error: response.error,
                }))
            }
            Some(Err(e)) => {
                error!("[DownloadStream] Stream error: {}", e);
                Some(Err(CommonError::IPCError { Description: 
 format!("Stream error: {}", e),
                }))
            }
            None => None,
        }
    }
}

// ============================================================================
// tonic::Request Helper
// ============================================================================

/// Helper trait for converting types to tonic::Request
trait IntoRequestExt {
    fn into_request(self) -> tonic::Request<Self>
    where
        Self: Sized,
    {
        tonic::Request::new(self)
    }
}

impl<T> IntoRequestExt for T {}
