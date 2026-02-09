//! # TelemetrySpans - OpenTelemetry Span Management
//!
//! Centralized span creation and management for RPC services.

#[cfg(feature = "Telemetry")]
use opentelemetry::{Key, KeyValue, global, trace::Tracer};

/// Create a span with standard attributes
#[cfg(feature = "Telemetry")]
pub fn CreateSpan(service_name:&str, operation:&str) -> opentelemetry::trace::Span {
	let tracer = global::tracer(service_name);
	let mut span = tracer.start(operation);
	span.set_attribute(KeyValue::new("service.name", service_name));
	span.set_attribute(KeyValue::new("service.operation", operation));
	span
}

#[cfg(feature = "Telemetry")]
pub fn SetSuccess(span:&mut opentelemetry::trace::Span, success:bool) {
	span.set_attribute(KeyValue::new("success", success));
}

#[cfg(feature = "Telemetry")]
pub fn SetDuration(span:&mut opentelemetry::trace::Span, duration_ms:u64) {
	span.set_attribute(KeyValue::new("duration_ms", duration_ms as i64));
}
