//! # TelemetryMetrics - OpenTelemetry Metrics Recording
//!
//! Standardized metrics for RPC services.

#[cfg(feature = "Telemetry")]
use opentelemetry::{
	KeyValue,
	global,
	metrics::{Counter, Histogram, Meter},
};

#[cfg(feature = "Telemetry")]
pub struct ServiceMetrics {
	meter:Meter,
}

#[cfg(feature = "Telemetry")]
impl ServiceMetrics {
	pub fn new(service_name:&str) -> Self { Self { meter:global::meter(service_name) } }

	pub fn create_counter(&self, name:&str) -> Counter<u64> { self.meter.u64_counter(name).build() }

	pub fn create_histogram(&self, name:&str) -> Histogram<u64> { self.meter.u64_histogram(name).build() }
}

#[cfg(not(feature = "Telemetry"))]
pub struct ServiceMetrics;

#[cfg(not(feature = "Telemetry"))]
impl ServiceMetrics {
	pub fn new(_service_name:&str) -> Self { Self }
}
