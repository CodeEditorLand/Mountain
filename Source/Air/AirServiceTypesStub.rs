//! Air integration type stubs. Twenty children: one per request/response
//! DTO, the placeholder `AirClientType::Struct`, and the
//! `DEFAULT_AIR_SERVER_ADDRESS` constant. Every `AirClientType` method
//! returns "feature not implemented" until the real `AirLibrary` client
//! lands behind `--features AirIntegration`.
//!
//! ## Status
//!
//! No callers as of 2026-05-02. Remove this entire module when the live
//! Air client is wired in.
//!
//! ## Sub-modules
//!
//! - [`AirClientType`]: Stub client implementation
//! - [`AirMetricsProtoDTO`]: Metrics response inner payload
//! - [`ApplyUpdateRequest`]: Apply update request DTO
//! - [`ApplyUpdateResponse`]: Apply update response DTO
//! - [`AuthenticationRequest`]: Authentication request DTO
//! - [`AuthenticationResponse`]: Authentication response DTO
//! - [`DEFAULT_AIR_SERVER_ADDRESS`]: Default server address constant
//! - [`DownloadFileResponse`]: Download file response DTO
//! - [`DownloadRequest`]: Download file request DTO
//! - [`FileResultProtoDTO`]: Search result file entry
//! - [`IndexFilesResponse`]: Index files response DTO
//! - [`IndexRequest`]: Index files request DTO
//! - [`MetricsRequest`]: Metrics request DTO
//! - [`MetricsResponse`]: Metrics response DTO
//! - [`SearchFilesResponse`]: Search files response DTO
//! - [`SearchRequest`]: Search files request DTO
//! - [`StatusRequest`]: Status request DTO
//! - [`StatusResponse`]: Status response DTO
//! - [`UpdateCheckRequest`]: Update check request DTO
//! - [`UpdateCheckResponse`]: Update check response DTO

/// Stub client type used while AirIntegration feature is off.
pub mod AirClientType;

/// Inner metrics payload for MetricsResponse.
pub mod AirMetricsProtoDTO;

/// Apply update request DTO.
pub mod ApplyUpdateRequest;

/// Apply update response DTO.
pub mod ApplyUpdateResponse;

/// Authentication request DTO.
pub mod AuthenticationRequest;

/// Authentication response DTO.
pub mod AuthenticationResponse;

/// Default Air server address (127.0.0.1:50051).
pub mod DEFAULT_AIR_SERVER_ADDRESS;

/// Download file response DTO.
pub mod DownloadFileResponse;

/// Download file request DTO.
pub mod DownloadRequest;

/// Single file result inside SearchFilesResponse.
pub mod FileResultProtoDTO;

/// Index files response DTO.
pub mod IndexFilesResponse;

/// Index files request DTO.
pub mod IndexRequest;

/// Metrics request DTO.
pub mod MetricsRequest;

/// Metrics response DTO.
pub mod MetricsResponse;

/// Search files response DTO.
pub mod SearchFilesResponse;

/// Search files request DTO.
pub mod SearchRequest;

/// Status request DTO.
pub mod StatusRequest;

/// Status response DTO with uptime, request counts, and health flag.
pub mod StatusResponse;

/// Update check request DTO.
pub mod UpdateCheckRequest;

/// Update check response DTO.
pub mod UpdateCheckResponse;
