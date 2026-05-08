// TraceLog - debug tracing macro for fine-grained execution step tracking.

/// Logs a checkpoint message at TRACE level (for "every step" tracing).
///
/// This macro provides a low-intrusion way to trace execution flow through
/// the application startup and shutdown sequences. It expands to a single
/// dev_log!("lifecycle", ) call which incurs zero overhead when TRACE logging
/// is disabled.
///
/// # Example
///
/// ```rust,ignore
/// TraceStep!("[Boot] [Runtime] Building Tokio runtime...");
/// TraceStep!("[Boot] [Setup] Configuration loaded: {}", ConfigPath);
/// ```
#[macro_export]
macro_rules! TraceStep {

	($($arg:tt)*) => {{

		dev_log!("lifecycle", $($arg)*);
	}};
}
