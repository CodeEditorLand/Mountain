//! # MappedEffect (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module defines the MappedEffect type alias, which is the type-erased
//! unit of work that the dispatch logic can execute.
//!
//! ## ARCHITECTURAL ROLE
//!
//! MappedEffect serves as the **effect abstraction** in Track's dispatch
//! system:
//!
//! ```text
//! Dispatch Logic ──► MappedEffect (Boxed Closure) ──► ApplicationRunTime Execution
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **MappedEffect**: Type alias for boxed async closure signature
//!
//! ## ERROR HANDLING
//!
//! - All effects return Result<Value, String> for IPC compatibility
//!
//! ## LOGGING
//!
//! - Logging is handled by individual effect implementations
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Boxed closure allocation is lightweight
//! - Async operations avoid blocking
//!
//! ## TODO
//!
//! - [ ] Consider implementing an effect pool to cache frequently created
//!   effects

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// A type alias for a boxed, runnable effect. This is the "type-erased" unit of
/// work that the dispatch logic can execute.
pub type MappedEffect =
	Box<dyn FnOnce(Arc<ApplicationRunTime>) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> + Send>;
