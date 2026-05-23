
//! Build a `tracing::Span` with structured attributes attached. Returns
//! `()` when the `Telemetry` feature is off so call sites don't need
//! their own `cfg` gates.

#[cfg(feature = "Telemetry")]
pub fn Fn(Name:&str, Attributes:&[(&str, &str)]) -> tracing::Span {
	let mut Span = tracing::span!(tracing::Level::INFO, Name);

	for (Key, Value) in Attributes {
		Span.record(*Key, *Value);
	}

	Span
}

#[cfg(not(feature = "Telemetry"))]
pub fn Fn(_Name:&str, _Attributes:&[(&str, &str)]) {}
