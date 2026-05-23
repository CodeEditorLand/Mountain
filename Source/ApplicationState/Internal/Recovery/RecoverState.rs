//! # RecoverState - Internal Recovery Utilities
//!
//! Three composable primitives the recovery flow uses to clean up
//! corrupted state without taking the runtime down:
//!
//! Layout (one export per file, file name = identity):
//! - `ValidateAndCleanState::Fn` - predicate-driven map filter with
//!   warn-on-removal logging.
//! - `SafeStateOperationWithTimeout::Fn` - off-thread blocking op with a hard
//!   timeout (the worker is allowed to finish in the background; only the
//!   receiver gives up).
//! - `RecoverStateWithBackoff::Fn` - async retry with exponential backoff (100
//!   ms, doubled per failure).
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. Wire into the
//! `ApplicationState/Internal/Recovery` flow once the recovery
//! triggers are formalised.

pub mod RecoverStateWithBackoff;

pub mod SafeStateOperationWithTimeout;

pub mod ValidateAndCleanState;
