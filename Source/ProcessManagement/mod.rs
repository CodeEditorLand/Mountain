//! # ProcessManagement Module
//!
//! This module contains the core logic for managing the lifecycle of external
//! sidecar processes, specifically the `Cocoon` extension host. It handles
//! spawning the process, establishing communication, and performing the
//! initial handshake.

#![allow(non_snake_case)]

pub mod CocoonManagement;
pub mod InitializationData;

// pub use self::CocoonManagement::*;
