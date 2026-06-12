//! # Shim
//!
//! Deep-hook interception layer for VS Code engine events at the Rust level.
//! Gated behind `TierShim` env var (default: `None` — zero overhead).
//!
//! ## Modules
//!
//! - `Gate` — Compile-time tier gate (reads `env!("TierShim")`)
//! - `SwallowMap` — Pattern-matching decision engine for IPC method routing
//!
//! ## 🟠 Orange (Low-Level) | 🔵 Blue (Coverage)
//!
//! | Tier | Color | Rust Feature | What |
//! |------|-------|-------------|------|
//! | `None` | — | (none) | All shim code dead-code-eliminated |
//! | `Proxy` | 🔵 | `shim` | Audit-only observation |
//! | `Replace` | 🔵 | `shim` | Individual service replacement |
//! | `Own` | 🟠 | `shim-own` | Container ownership |
//! | `Preempt` | 🟠 | `shim-preempt` | Full engine control |
//!
//! See: `.hermes/microsoft/05-Mountain-Rust-Intercept.md`

pub mod Gate;
pub mod SwallowMap;
