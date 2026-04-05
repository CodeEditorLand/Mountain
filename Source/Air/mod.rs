//! # Air (Air Integration Module)
//!
//! RESPONSIBILITIES:
//! - Provides gRPC client connectivity to the Air daemon service
//! - Implements Air service methods for:
//!   - Update management and distribution
//!   - Authentication and credential management
//!   - File indexing and search operations
//!   - System monitoring and metrics collection
//! - Handles connection management and error translation to `CommonError`
//! - Wraps client in `Arc` for shared access across the application
//!
//! ARCHITECTURAL ROLE:
//! - Integration point with the Air background service (daemon)
//! - Used by multiple Mountain components:
//! - `UpdateService` for self-updates
//!   - [`SearchProvider`] for file search
//!   - [`SecretProvider`] for secret
//!     storage
//! - Connection is optional; Mountain can function without Air (graceful
//!   degradation)
//! - Service discovery and health checking via gRPC
//!
//! MODULE STRUCTURE:
//! - `AirClient` - gRPC client wrapper with connection management
//! - `AirServiceProvider` - high-level provider with automatic request ID
//!   generation
//! - `AirServiceTypesStub` - stub types for when Air library is unavailable
//!   (legacy)
//!
//! CONNECTION PATTERNS:
//! - Uses tonic gRPC client for transport
//! - Connection establishment via `connect()` method
//! - Health checking with timeout protection
//! - Thread-safe operations via `Arc<AirClient>`
//!
//! ERROR HANDLING:
//! - All gRPC errors translated to
//!   [`CommonError::IPCError`](CommonLibrary::Error::CommonError)
//! - Connection failures logged and return error
//! - Service unavailability handled gracefully (return error, caller decides
//!   fallback)
//!
//! PERFORMANCE:
//! - gRPC channels are expensive; reuse via `Arc<AirClient>`
//! - Non-blocking async operations via tokio
//! - Request ID generation for tracing
//!
//! VS CODE REFERENCE:
//! - `vs/platform/telemetry/common/telemetry.ts` - telemetry/analytics service
//!   pattern
//! - `vs/platform/update/common/update.ts` - update service integration
//! - `vs/workbench/services/search/common/search.ts` - search service
//!   architecture
//!
//! TODO:
//! - Implement connection retry with exponential backoff
//! - Add connection pooling for multiple concurrent requests
//! - Implement request caching for frequently accessed data (auth tokens, etc.)
//! - Add metrics collection for Air service calls (latency, success rate,
//!   errors)
//! - Implement fallback strategies when Air unavailable (local search, etc.)
//! - Support for multiple Air daemons (load balancing/failover)
//! - Add request timeout configuration (configurable per operation type)
//! - Implement request/response logging for debugging
//! - Add telemetry for Air service health and usage
//! - Implement bidirectional streaming for real-time updates
//!
//! MODULE CONTENTS:
//! - Re-exports: `AirClient`, `AirServiceProvider`, response types, and helper
//!   functions

// Module sub-modules
pub mod AirClient;
pub mod AirServiceProvider;

// Access AirClient struct as: crate::Air::AirClient::AirClientImpl
// Re-exports using module prefix to avoid naming conflicts
pub use AirClient::{
	AirClient as Client,
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
// Re-export the original name for compatibility (using type alias inside the module)
pub use AirServiceProvider::generate_request_id;

// Note: AirServiceProvider struct is available via
// crate::Air::AirServiceProvider::AirServiceProvider

// Stub types for Air integration when AirLibrary is not available (legacy)
// Note: These are kept for backward compatibility but should not be used in new
// code
#[deprecated(note = "Use AirClient and AirServiceProvider instead")]
pub mod AirServiceTypesStub;
