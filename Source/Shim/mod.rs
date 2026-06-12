//! # Shim
//!
//! Deep-hook interception layer for VS Code engine events at the Rust level.
//! Gated behind `TierShim` env var (default: `None` — zero overhead).
//!
//! ## Modules
//!
//! - `Gate` — Compile-time tier gate (reads `env!("TierShim")`)
//! - `SwallowMap` — Pattern-matching engine for IPC method routing

pub mod Gate;

pub mod NativeBus;

pub mod SwallowMap;
