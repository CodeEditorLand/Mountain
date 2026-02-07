//! # Execute (RunTime)
//!
//! ## RESPONSIBILITIES
//!
//! Effect execution logic that bridges the declarative ActionEffect system
//! with the Echo scheduler for high-performance task execution.
//!
//! Provides core execution methods through impl blocks on ApplicationRunTime:
//! - Basic effect execution through Echo scheduler
//! - Timeout-based execution with cancellation
//! - Retry mechanisms with exponential backoff
//!
//! ## ARCHITECTURAL ROLE
//!
//! The execution engine in Mountain's architecture that handles the "how"
//! of effect execution, while ActionEffect defines the "what".
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Core effect execution functions implemented on ApplicationRunTime
//!
//! ## ERROR HANDLING
//!
//! All errors are propagated through Result<T, E> with detailed context.
//! Effect errors are converted to CommonError when appropriate.
//! Timeouts return timeout-specific errors.
//! Retry failures include attempt count and last error information.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels:
//! - `info`: Effect submission and completion
//! - `debug`: Detailed execution steps
//! - `warn`: Retry attempts and recoverable errors
//! - `error`: Failed operations and timeout occurrences
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Uses oneshot channels for result collection (minimal overhead)
//! - Tasks are submitted to Echo's work-stealing scheduler
//! - Timeout uses tokio::time::timeout for efficient cancellation
//! - Retry with exponential backoff prevents system overload
//!
//! ## TODO
//!
//! Medium Priority:
//! - [ ] Add effect execution metrics (count, duration, success rate)
//! - [ ] Implement effect execution tracing and profiling
//! - [ ] Add effect prioritization levels beyond Normal
//! - [ ] Implement effect cancellation token propagation
//! - [ ] Add circuit breaker pattern for failing effects
//!
//! Low Priority:
//! - [ ] Implement distributed effect execution across instances
//! - [ ] Add effect result caching for idempotent operations
//! - [ ] Implement effect pipeline with chaining and composition
//! - [ ] Add real-time effect monitoring dashboard
//! - [ ] Implement adaptive timeout based on historical performance
//! - [ ] Add effect execution limits (rate limiting)

// --- Sub-modules ---

/// Effect execution functions implemented on ApplicationRunTime.
pub mod Fn;

// Note: The execution functions (Run, RunWithTimeout, RunWithRetry) are now
// implemented as methods on the ApplicationRunTime struct in Fn.rs
