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
//!   - `SearchProvider` for file search
//!   - `SecretProvider` for secret storage
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
//! ## Planned Work
//!
//! - Connection retry with exponential backoff
//! - Connection pooling for concurrent requests
//! - Request caching for frequently accessed data (auth tokens)
//! - Metrics collection for Air service calls (latency, success rate, errors)
//! - Fallback strategies when Air is unavailable (local search)
//! - Support for multiple Air daemons (load balancing/failover)
//! - Configurable request timeouts per operation type
//! - Request/response logging for debugging
//! - Telemetry for Air service health and usage
//! - Bidirectional streaming for real-time updates

//! MODULE CONTENTS:
//! - Re-exports: `AirClient`, `AirServiceProvider`, response types, and helper
//!   functions

// Module sub-modules
pub mod AirClient;

pub mod AirServiceProvider;

// Stub types for Air integration when AirLibrary is not available (legacy)
