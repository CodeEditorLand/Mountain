//! # ValidatePermission
//!
//! Role-based access control for IPC operations. Two atoms:
//! `SecurityContext::Struct` - the per-request envelope
//! (user / roles / permissions / IP / timestamp), and
//! `Validator::Struct` - the engine that holds the role +
//! permission tables, the operation → permissions map, and
//! enforces the default-deny policy through
//! `Validator::Struct::ValidatePermission`.

pub mod SecurityContext;

pub mod Validator;
