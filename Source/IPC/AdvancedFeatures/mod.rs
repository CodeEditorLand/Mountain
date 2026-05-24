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

pub mod MountainCreateCollaborationSession;

pub mod MountainGetCacheStats;

pub mod MountainGetCollaborationSessions;

pub mod MountainGetPerformanceStats;
