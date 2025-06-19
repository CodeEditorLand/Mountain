//! # Vine gRPC Server Module
//!
//! This module contains the implementation for the Mountain gRPC server. It is
//! responsible for listening for incoming connections from sidecars like
//! `Cocoon`, handling RPC requests, and dispatching them into the Mountain
//! application logic.

#![allow(non_snake_case)]

pub mod Initialize;
pub mod MountainVinegRPCService;

// /// The primary public function for this module, responsible for creating and
// /// starting the gRPC server on a background task.
// pub use self::Initialize::Initialize;
