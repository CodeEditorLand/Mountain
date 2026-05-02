#![allow(non_snake_case)]

//! OTEL telemetry RPC. `TelemetryService::Struct` is the impl handle;
//! `TraceSpan::Struct` and `ServiceMetrics::Struct` are the wire DTOs.

pub mod ServiceMetrics;
pub mod TelemetryService;
pub mod TraceSpan;
