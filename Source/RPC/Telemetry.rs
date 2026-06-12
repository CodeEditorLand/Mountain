//! OTEL telemetry RPC. `TelemetryService::Struct` is the impl handle;
//! `TraceSpan::Struct` and `ServiceMetrics::Struct` are the wire DTOs.
/// Service metrics DTO: captures name, count, and sum for a single metric
/// snapshot.
pub mod ServiceMetrics;

/// Telemetry service: routes OTEL trace and metric submission from the
/// extension host.
pub mod TelemetryService;

/// Trace span DTO: models a single OTEL span with ID, parent, timing, and name.
pub mod TraceSpan;
