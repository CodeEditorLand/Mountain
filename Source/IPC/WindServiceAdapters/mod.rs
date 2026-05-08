#![allow(non_snake_case)]

//! # Wind service adapters
//!
//! Mountain → Wind bridge: takes Mountain's sandbox config
//! and runtime providers, exposes them in Wind's expected
//! shape (`WindDesktopConfiguration::Struct`,
//! `WindFileService::Struct`, `WindStorageService::Struct`,
//! `WindConfigurationService::Struct`,
//! `WindEnvironmentService::Struct`). The
//! `WindServiceAdapter::Struct` factory holds the runtime
//! handle and produces the per-domain wrappers on demand.

pub mod FileToDiff;

pub mod FileToOpenOrCreate;

pub mod FilesToWait;

pub mod Logger;

pub mod MountainSandboxConfiguration;

pub mod OsInfo;

pub mod Profiles;

pub mod WindConfigurationService;

pub mod WindDesktopConfiguration;

pub mod WindEnvironmentService;

pub mod WindFileService;

pub mod WindServiceAdapter;

pub mod WindStorageService;
