//! # Permission Manager (IPC Security)
//!
//! Role-based access control for the IPC layer with built-in
//! audit logging. `Manager::Struct` is the enforcement core;
//! `SecurityContext::Struct` is the per-request envelope;
//! `SecurityEvent::Struct` + `SecurityEventType::Enum` carry
//! the audit trail.

pub mod Manager;

pub mod SecurityContext;

pub mod SecurityEvent;

pub mod SecurityEventType;
