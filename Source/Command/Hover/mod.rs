//! # Hover Module
//!
//! Atomic module for hover language feature command.
//!
//! ## Structure
//!
//! - `Interface.rs` - Types and traits for hover
//! - `Fn.rs` - Implementation of hover functionality
//!
//! ## Usage
//!
//! ```rust
//! use crate::Command::Hover::{Fn::Hover, Interface::HoverResponse};
//! ```

pub mod Fn;
pub mod Interface;

// Re-export for convenience
pub use Fn::Hover;
pub use Interface::{HoverContent, HoverRequest, HoverResponse, Position, Range};
