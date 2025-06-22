//! # Vine gRPC Module
//!
//! This module encapsulates all logic related to the gRPC-based
//! Inter-Process Communication (IPC) system, codenamed "Vine". It manages both
//! the server for listening to `Cocoon` and the client for sending messages to
//! `Cocoon`.

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---
pub mod Client;

pub mod Error;

pub mod Generated;

pub mod Server;
