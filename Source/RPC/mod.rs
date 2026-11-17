//! # RPC
//!
//! Mountain RPC services. `CocoonService` is the active surface - the tonic
//! server implementation that the extension host connects to.

/// CocoonService: tonic gRPC server exposing the full extension-host API
/// surface.
pub mod CocoonService;

/// Command-registration service and its DTOs.
pub mod Commands;

/// Configuration service and its scope/update DTOs.
pub mod Configuration;

/// Debug-Adapter-Protocol handle (gated behind `debug-protocol`).
pub mod Debug;

/// Extension-host submission gate over Echo's work-stealing scheduler.
pub mod EchoAction;

/// Child-process handle (gated behind `child-processes`).
pub mod Processes;

/// Source-control-management handle (gated behind `scm-support`).
pub mod SCM;

/// OTEL telemetry service and its span/metric DTOs.
pub mod Telemetry;

/// Terminal-emulation handle (gated behind `terminals`).
pub mod Terminals;

/// Shared request/response envelopes.
pub mod Types;

/// Vine connection DTOs for cross-service use.
pub mod Vine;

/// Window-management handle (gated behind `grove` or `cocoon`).
pub mod Windows;

/// Workspace service and its folder/document DTOs.
pub mod Workspace;
