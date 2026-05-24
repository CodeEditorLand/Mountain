//! `AirServiceProvider` - atomized.

pub mod Address;
pub mod ApplyUpdate;
pub mod Authenticate;
pub mod CheckForUpdates;
pub mod Client;
pub mod DownloadFile;
pub mod DownloadStream;
pub mod DownloadUpdate;
pub mod FromClient;
pub mod GenerateRequestID;
pub mod GetConfiguration;
pub mod GetFileInfo;
pub mod GetMetrics;
pub mod GetResourceUsage;
pub mod GetStatus;
pub mod HealthCheck;
pub mod IndexFiles;
pub mod IsConnected;
pub mod New;
pub mod NewDefault;
pub mod SearchFiles;
pub mod SetResourceLimits;
pub mod UpdateConfiguration;

// # AirServiceProvider
//
// High-level API surface for Air service methods.
//
// ## RESPONSIBILITIES
//
// - **Service Facade**: Provide convenient, high-level interface to Air daemon
// - **Authentication**: Manage user authentication and credentials
// - **Updates**: Check for and download application updates
// - **File Indexing**: Query Air's file search and indexing capabilities
// - **System Monitoring**: Retrieve system metrics and health data
// - **Graceful Degradation**: Handle Air unavailability with fallbacks
//
// ## ARCHITECTURAL ROLE
//
// AirServiceProvider acts as a facade over the raw `AirClient`, providing:
// - Simplified API for common operations
// - Automatic error handling and translation
// - Request ID generation for tracing
// - Connection state management
//
// ```text
// Application ──► AirServiceProvider ──► AirClient ──► gRPC ──► Air Daemon
// ```
//
// ### Dependencies
// - `AirClient`: Low-level gRPC client
// - `uuid`: For generating request identifiers
// - `CommonLibrary::Error::CommonError`: Error types
//
// ### Dependents
// - `Binary::Service::VineStart`: Initializes Air service
// - `MountainEnvironment`: Can delegate to Air when available
//
// ## IMPLEMENTATION
//
// This implementation provides a fully functional provider that wraps the
// AirClient type with automatic request ID generation and error handling.
//
// ## ERROR HANDLING
//
// All operations return `Result<T, CommonError>` with:
// - Translated gRPC errors to appropriate CommonError types
// - Request IDs included in logs for tracing
// - Graceful fallback to local operations when Air is unavailable
//
// ## PERFORMANCE
//
// - Request ID generation uses UUID v4 (cryptographically random)
// - Thread-safe operations via `Arc<AirClient>`
// - Non-blocking async operations via tokio
//
// ## VSCODE REFERENCE
//
// Patterns borrowed from VS Code:
// - `vs/platform/update/common/updateService.ts` - Update management
// - `vs/platform/authentication/common/authenticationService.ts` - Auth
//   handling
// - `vs/platform/filesystem/common/filesystem.ts` - File indexing
//
// ## MODULE CONTENTS
//
// - [`AirServiceProvider`]: Main provider struct
// - [`generate_request_id`]: Helper function for UUID generation

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::Error::CommonError::CommonError;

use super::AirClient::{Struct,
	AirMetrics,
	AirStatus,
	DEFAULT_AIR_SERVER_ADDRESS,
	DownloadStream,
	DownloadStreamChunk,
	ExtendedFileInfo,
	FileInfo,
	FileResult,
	IndexInfo,
	ResourceUsage,
	UpdateInfo,
};
use crate::{Air::AirServiceProvider::GenerateRequestID::Fn as generate_request_id, dev_log};

pub mod GenerateRequestID;

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
/// use Mountain::Air::AirServiceProvider::{Struct, DEFAULT_AIR_SERVER_ADDRESS};
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
pub struct Struct {
	/// The underlying Air client wrapped in Arc for thread safety
	client:Arc<AirClient>,
}
