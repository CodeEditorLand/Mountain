//! Air-integration type stubs. Twenty children: one per request/response
//! DTO, the placeholder `AirClientType::Struct`, and the
//! `DEFAULT_AIR_SERVER_ADDRESS::Const` server-address constant. Every
//! `AirClientType::Struct` method returns "feature not implemented" until
//! the real `AirLibrary` client lands behind `--features AirIntegration`.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. Remove this entire module when
//! the live Air client is wired in.

/// Airclienttype module.
pub mod AirClientType;

/// Airmetricsprotodto module.
pub mod AirMetricsProtoDTO;

/// Applyupdaterequest module.
pub mod ApplyUpdateRequest;

/// Applyupdateresponse module.
pub mod ApplyUpdateResponse;

/// Authenticationrequest module.
pub mod AuthenticationRequest;

/// Authenticationresponse module.
pub mod AuthenticationResponse;

/// Default air server address module.
pub mod DEFAULT_AIR_SERVER_ADDRESS;

/// Downloadfileresponse module.
pub mod DownloadFileResponse;

/// Downloadrequest module.
pub mod DownloadRequest;

/// Fileresultprotodto module.
pub mod FileResultProtoDTO;

/// Indexfilesresponse module.
pub mod IndexFilesResponse;

/// Indexrequest module.
pub mod IndexRequest;

/// Metricsrequest module.
pub mod MetricsRequest;

/// Metricsresponse module.
pub mod MetricsResponse;

/// Searchfilesresponse module.
pub mod SearchFilesResponse;

/// Searchrequest module.
pub mod SearchRequest;

/// Statusrequest module.
pub mod StatusRequest;

/// Statusresponse module.
pub mod StatusResponse;

/// Updatecheckrequest module.
pub mod UpdateCheckRequest;

/// Updatecheckresponse module.
pub mod UpdateCheckResponse;
