//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # Track Module
//!
//! ## Responsibilities
//!
//! This module acts as the central request dispatcher for the Mountain
//! application. It is the primary entry point for all incoming commands and RPC
//! calls, whether they originate from the `Sky` frontend or a `Cocoon`
//! sidecar.
//!
//! ### Core Functions:
//! - **Request Routing**: Route all incoming requests from frontend and sidecars
//! - **Effect Creation**: Transform string-based command/RPC names into strongly-typed ActionEffects
//! - **Command Dispatching**: Execute effects through the ApplicationRunTime
//! - **Error Handling**: Provide comprehensive error handling and recovery
//! - **Type Safety**: Ensure type-safe message passing through Rust's type system
//!
//! ## Architectural Role
//!
//! The Track module serves as the **routing layer** in Mountain's architecture:
//!
//! ```text
//! Sky (Frontend) ──► Track (Router) ──► ApplicationRunTime (Executor) ──► Services
//! Cocoon (Sidecar) ──► Track (Router) ──► ApplicationRunTime (Executor) ──► Providers
//! ```
//!
//! ### Design Principles:
//! 1. **Declarative Effects**: Commands create declarative ActionEffects that describe "what"
//!    needs to happen, not "how"
//! 2. **Type-Safe Dispatch**: String-based method names are mapped to strongly-typed effects
//! 3. **Performance-Critical**: Direct provider calls are used for hot paths
//! 4. **Comprehensive Logging**: All routing decisions are logged for observability
//!
//! ## Key Components
//!
//! - **EffectCreation**: Creates ActionEffects from request payloads
//! - **DispatchLogic**: Main dispatch functions for routing requests
//!
//! ## TODOs
 //!
 // High Priority:
 // - [ ] Add metrics/telemetry for dispatch latency
 // - [ ] Implement command caching for frequently used effects
 // - [ ] Add circuit breaker pattern for failing provider calls
 //!
 // Medium Priority:
 // - [ ] Add request rate limiting per client
 // - [ ] Implement command batching for related operations
 // - [ ] Add warm-up phase for critical paths
 //!
 // Low Priority:
 // - [ ] Add request tracing across the entire pipeline
 // - [ ] Implement request replay for debugging
 // - [ ] Add command versioning for backwards compatibility

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---

/// Contains the main dispatch functions.
pub mod DispatchLogic;

/// Contains the logic for creating `ActionEffect`s from request payloads.
pub mod EffectCreation;
