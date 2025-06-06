// File: Rpc/Argument/Common/mod.rs
// This module defines common Data Transfer Objects (DTOs) that might be
// shared across various RPC argument structures, such as glob patterns.

mod GlobPattern; // Renamed from Globpattern

pub use GlobPattern::GlobPattern;
