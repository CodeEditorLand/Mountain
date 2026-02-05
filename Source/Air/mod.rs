//! # Air (Air Integration Module)
//!
//! RESPONSIBILITIES:
//! - Provides gRPC client connectivity to the Air daemon service
//! - Implements Air service methods for:
//!   - Update management and distribution
//!   - Authentication and credential management
//!   - File indexing and search operations
//!   - System monitoring and metrics collection
//! - Handles connection management and error translation to [`CommonError`]
//! - Wraps client in [`Arc`] for shared access across the application
//!
//! ARCHITECTURAL ROLE:
//! - Integration point with the Air background service (daemon)
//! - Used by multiple Mountain components:
//!   - [`UpdateService`](crate::Update::UpdateService) for self-updates
//!   - [`SearchProvider`](crate::Environment::SearchProvider) for file search
//!   - [`AuthenticationProvider`] (if implemented) for user credentials
//! - Connection is optional; Mountain can function without Air (graceful
//!   degradation)
//! - Service discovery and health checking via gRPC
//!
//! MODULE STRUCTURE:
//! - `AirClient` - gRPC client wrapper with connection management
//! - `AirServiceProvider` - provider trait implementation for DI
//! - `AirServiceTypesStub` - stub types for when Air library is unavailable
//!
//! CONNECTION PATTERNS:
//! - Uses tonic gRPC client for transport
//! - Connection establishment via `ConnectToSideCar` (host:port)
//! - Health checking with timeout protection
//! - TODO: Implement connection pooling, retry with exponential backoff
//!
//! ERROR HANDLING:
//! - All gRPC errors translated to
//!   [`CommonError::IPCError`](CommonLibrary::Error::CommonError)
//! - Connection failures logged and return error
//! - Service unavailability handled gracefully (return error, caller decides
//!   fallback)
//!
//! PERFORMANCE:
//! - gRPC channels are expensive; reuse via Arc<AirServiceClient>
//! - TODO: Add request caching for frequently accessed data (auth tokens, etc.)
//! - TODO: Implement metrics collection for Air service calls (latency, success
//!   rate)
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
//! - Re-exports: `AirClient`, `AirServiceProvider`, `AirServiceTypesStub`

pub mod AirClient;
pub mod AirServiceProvider;

// Stub types for Air integration when AirLibrary is not available
pub mod AirServiceTypesStub;
