// File: Mountain/Source/Air/mod.rs
// Role: Air gRPC client module for Mountain
// Responsibilities:
//   - Provide gRPC client connectivity to the Air daemon service
//   - Implement Air service methods for update management, authentication, file
//     indexing, and system monitoring
//   - Handle connection management and error translation to CommonError
//   - Wrap client in Arc for shared access across the application

//! # Air Integration Module
//!
//! ## RESPONSIBILITY
//! - gRPC client for Air daemon (background service)
//! - Update management and distribution
//! - Authentication and credential management
//! - File indexing and search operations
//! - System monitoring and metrics collection
//! - Connection pooling and health checks
//!
//! ## ARCHITECTURAL ROLE
//! - Provides Air daemon integration to Mountain
//! - Integrates with UpdateService for self-updates
//! - Provides authentication to Environment providers
//! - Supplies search capabilities to Environment/SearchProvider
//! - Handles Air service unavailability gracefully
//!
//! ## DESIGN PATTERNS (Borrowed from VSCode)
//! - Background service integration pattern
//! - gRPC client connection management
//! - Service provider pattern with availability checking
//! - Graceful degradation when service unavailable
//!
//! ## TODO
//! - Implement connection retry with backoff
//! - Add connection pooling for multiple concurrent requests
//! - Implement request caching for frequently accessed data
//! - Add metrics collection for Air service calls
//! - Implement fallback strategies when Air unavailable
//! - Support for multiple Air daemons (load balancing)
//! - Add request timeout configuration
//! - Implement request/response logging

pub mod AirClient;
pub mod AirServiceProvider;

// Stub types for Air integration when AirLibrary is not available
pub mod AirServiceTypesStub;
