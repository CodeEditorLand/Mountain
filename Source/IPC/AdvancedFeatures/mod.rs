#![allow(non_snake_case)]

//! # Advanced IPC features
//!
//! Performance counters, message-cache, realtime collaboration
//! session tracking, and the four `mountain_*` Tauri commands
//! that surface them. The `Features::Struct` aggregator + impl
//! lives in `Features.rs` (tightly-coupled cluster); the DTOs
//! and Tauri commands live in their own siblings.

pub mod CachedMessage;
pub mod CollaborationPermissions;
pub mod CollaborationSession;
pub mod Features;
pub mod InitializeAdvancedFeatures;
pub mod MessageCache;
pub mod PerformanceStats;
pub mod mountain_create_collaboration_session;
pub mod mountain_get_cache_stats;
pub mod mountain_get_collaboration_sessions;
pub mod mountain_get_performance_stats;
