#![allow(non_snake_case)]

//! # Service Discovery and Information
//!
//! Tracks every service Mountain talks to: its lifecycle state,
//! performance counters, dependency graph, and (optionally) a network
//! endpoint. Used by the in-process service registry to make health-
//! based routing decisions.
//!
//! Layout (one export per file, file name = identity):
//! - `ServiceState::Enum` - Running / Degraded / Stopped / Error / Starting /
//!   ShuttingDown.
//! - `ServicePerformance::Struct` - request/error counters + rolling mean
//!   response latency.
//! - `ServiceEndpoint::Struct` - protocol/host/port (+ UDS path).
//! - `ServiceInfo::Struct` - the per-service descriptor.
//! - `ServiceRegistry::Struct` - discovery-cadence-aware map keyed by service
//!   name.
//!
//! TODO: zero callers as of 2026-05-02. Pending wire-up from the
//! gRPC and Tauri IPC dispatch hot paths.

pub mod ServiceEndpoint;

pub mod ServiceInfo;

pub mod ServicePerformance;

pub mod ServiceRegistry;

pub mod ServiceState;
