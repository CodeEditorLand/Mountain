//! # Vine gRPC Server Module
//!
//! This module contains the implementation for the Mountain gRPC server. It is
//! responsible for listening for incoming connections from sidecars like
//! `Cocoon`, handling RPC requests, and dispatching them into the Mountain
//! application logic.

#![allow(non_snakeCase)]

pub mod Initialize;

pub mod MountainVinegRPCService;

pub mod CocoonServiceServer;
