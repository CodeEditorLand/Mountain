

/**
 * @module vine
 * @description This module encapsulates all logic related to the gRPC-based
 * Inter-Process Communication (IPC) system, codenamed "Vine". It manages both
 * the server for listening to Cocoon and the client for sending messages to Cocoon.
 */

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---

/// Contains the logic for the gRPC client that connects to the Cocoon sidecar.
pub mod client;

/// Contains the logic for the gRPC server that listens for requests from Cocoon.
pub mod server;

/// Defines error types specific to the Vine IPC system.
mod error;

/// Includes and re-exports the code auto-generated from `vine.proto`.
mod generated;

// --- Public Re-exports ---

/**
 * The primary error type for all Vine operations.
 * @see error::VineError
 */
pub use self::error::VineError;

/**
 * The auto-generated gRPC client for the `CocoonService`.
 * @see generated::CocoonServiceClient
 */
pub use self::generated::CocoonServiceClient;

/**
 * The auto-generated gRPC server trait for the `MountainService`.
 * Our `MountainVineGrpcService` will implement this trait.
 * @see generated::MountainServiceServer
 */
pub use self::generated::MountainServiceServer;
