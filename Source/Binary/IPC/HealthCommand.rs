#![allow(non_snake_case)]

//! # HealthCommand - Wind SharedProcessProxy bridge
//!
//! Tauri commands invoked directly by Wind's `SharedProcessProxy`
//! health-check pings. Each is a thin probe that maps the renderer's
//! abstract service identifier onto Mountain's actual readiness state.
//!
//! Layout (one Tauri command per file, snake_case wire-bound names per
//! the Naming-Convention exception):
//! - `cocoon_extension_host_health::cocoon_extension_host_health`
//! - `cocoon_search_service_health::cocoon_search_service_health`
//! - `cocoon_debug_service_health::cocoon_debug_service_health`
//! - `shared_process_service_health::shared_process_service_health`

pub mod cocoon_debug_service_health;

pub mod cocoon_extension_host_health;

pub mod cocoon_search_service_health;

pub mod shared_process_service_health;
