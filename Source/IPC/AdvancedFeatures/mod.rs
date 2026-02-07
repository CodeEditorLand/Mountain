//! # Advanced Features Module (IPC)
//!
//! ## RESPONSIBILITIES
//! This module provides advanced IPC features including real-time collaboration
//! support, intelligent caching, and performance monitoring for the IPC layer.
//!
//! ## ARCHITECTURAL ROLE
//! This module extends the IPC capabilities with enhanced features for improved
//! user experience and performance.
//!
//! ## KEY COMPONENTS
//!
//! - **Features**: Main AdvancedFeatures orchestrator
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//!
//! ## LOGGING
//! Info-level for lifecycle events, debug for operations, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Caching with TTL for redundancy reduction
//! - Background tasks for monitoring and cleanup
//! - Efficient data structures for performance tracking
//!
//! ## TODO
//! - Add LRU cache eviction
//! - Implement predictive caching
//! - Add cursor position sharing
//! - Implement conflict resolution

pub mod Features;

pub use Features::{
	AdvancedFeatures,
	initialize_advanced_features,
	CollaborationSession,
	CollaborationPermissions,
	PerformanceStats,
	MessageCache,
	CachedMessage,
};
