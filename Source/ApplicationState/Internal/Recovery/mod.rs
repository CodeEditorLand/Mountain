//! # Recovery Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Provides recovery utilities for state validation and error recovery.
//!
//! ## ARCHITECTURAL ROLE
//! Recovery is part of the **Internal** module, providing
//! recovery utilities.
//!
//! ## KEY COMPONENTS
//! - RecoverState: State recovery utilities
//!
//! ## ERROR HANDLING
//! - Validates state data
//! - Implements recovery with backoff
//! - Timeout handling
//!
//! ## LOGGING
//! Operations are logged at appropriate levels.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient validation
//! - Exponential backoff for retries
//! - Timeout protection
//!
//! ## TODO
//! - [ ] Add state validation rules
//! - [ ] Implement checkpoint recovery
//! - [ ] Add recovery metrics

pub mod RecoverState;

pub use RecoverState::*;
