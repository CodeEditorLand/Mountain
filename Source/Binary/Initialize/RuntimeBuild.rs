#![allow(non_snake_case, unused_imports)]

//! # RuntimeBuild - Echo scheduler bring-up
//!
//! Constructs the Echo async scheduler with the right worker count and
//! telemetry knobs for the active build profile.
//!
//! Layout (one export per file, file name = identity):
//! - `SchedulerConfig::Struct` - tuning knobs with profile-aware `Default`.
//! - `CreateBuilder::Fn` - `SchedulerConfig` → `SchedulerBuilder`.
//! - `Build::Fn` - default bring-up; CPU count workers.
//! - `BuildWithConfig::Fn` - bring-up from a custom `SchedulerConfig::Struct`.
//! - `BuildDebug::Fn` - single-worker scheduler under the `Debug` feature.
//!
//! Profile cheat sheet:
//! - **Debug**: 1 worker, verbose logs.
//! - **Development**: CPU count workers, info logs.
//! - **Release**: CPU count workers, warn logs, telemetry.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. `Binary::Main` currently creates the
//! scheduler inline. Wire through this module when the profile-aware
//! build sites land.

pub mod Build;

pub mod BuildDebug;

pub mod BuildWithConfig;

pub mod CreateBuilder;

pub mod SchedulerConfig;
