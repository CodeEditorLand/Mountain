//! # HealthCommand - Wind SharedProcessProxy bridge
//!
//! Tauri commands invoked directly by Wind's `SharedProcessProxy`
//! health-check pings. Each is a thin probe that maps the renderer's
//! abstract service identifier onto Mountain's actual readiness state.
//!
//! Layout (one Tauri command per file, snake_case wire-bound names per
//! the Naming-Convention exception):
//! - `CocoonExtensionHostHealth::Fn`
//! - `CocoonSearchServiceHealth::Fn`
//! - `CocoonDebugServiceHealth::Fn`
//! - `SharedProcessServiceHealth::Fn`

pub mod CocoonDebugServiceHealth;

pub mod CocoonExtensionHostHealth;

pub mod CocoonSearchServiceHealth;

pub mod SharedProcessServiceHealth;
