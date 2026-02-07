//! # RunTime Module
//!
//! ## RESPONSIBILITIES
//!
//! Effect execution engine for the Mountain application. This module provides
//! the runtime environment for executing effects through the Echo scheduler.
//!
//! ### Core Functions:
//! - **Effect Execution**: Execute effects through the Echo scheduler
//! - **Task Management**: Submit tasks to the async runtime
//! - **Environment Integration**: Provide access to MountainEnvironment
//! - **Result Handling**: Collect and return effect results
//! - **Lifecycle Management**: Orchestrate graceful shutdown of all services
//! - **Error Recovery**: Comprehensive error handling and recovery mechanisms
//!
//! ## Architectural Role
//!
//! The RunTime module is the **execution engine** in Mountain's architecture:
//!
//! ```text
//! Track (Router) ──► RunTime (Executor) ──► Eco:Scheduler ──► Providers
//! Command ─────────────────────────────────────────────────────────┘
//! ```
//!
//! ### Design Principles:
//! 1. **Async-First**: All execution happens in an async context
//! 2. **Effect-Based**: Effects describe "what" to do, runtime determines "how"
//! 3. **Non-Blocking**: Uses Echo's work-stealing scheduler
//! 4. **Type-Safe**: Effect execution is type-safe through generics
//! 5. **Error Recovery**: Continues shutdown even when services fail
//! 6. **Graceful Degradation**: Provides fallback strategies for service unavailability
//!
//! ## Key Components
//!
//! - **ApplicationRunTime**: Main runtime struct and orchestration
//! - **Execute**: Effect execution logic (Run, RunWithTimeout, RunWithRetry)
//! - **Shutdown**: Service shutdown and lifecycle management
//!
//! ## TODOs
//! Medium Priority:
//! - [ ] Add effect timeout handling
//! - [ ] Implement effect execution metrics
//! - [ ] Add effect cancellation support
//! - [ ] Add effect tracing
//! - [ ] Implement effect prioritization
//!
//! Low Priority:
//! - [ ] Add effect execution limits (rate limiting)
//! - [ ] Implement distributed effect execution across instances
//! - [ ] Add effect result caching for idempotent operations
//! - [ ] Implement effect pipeline with chaining and composition

// --- Sub-modules ---

/// Application runtime module containing the struct definition.
/// The struct is accessible as `RunTime::ApplicationRunTime::ApplicationRunTime`.
pub mod ApplicationRunTime;

/// Effect execution logic.
pub mod Execute;

/// Service shutdown and lifecycle management.
pub mod Shutdown;
