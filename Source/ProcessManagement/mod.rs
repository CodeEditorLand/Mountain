//! # Process Management Module
//!
//! This module provides comprehensive lifecycle management for external sidecar
//! processes within the Mountain editor ecosystem. It is responsible for:
//!
//! ## Core Responsibilities
//!
//! 1. **Process Spawning**: Launching and configuring sidecar processes with
//!    proper environment variables, IPC channels, and communication endpoints
//!
//! 2. **Lifecycle Management**: Monitoring process health, handling automatic
//!    restarts on failure, managing graceful shutdowns, and tracking process
//!    state
//!
//! 3. **Communication Setup**: Establishing IPC connections via gRPC/Vine,
//!    performing initial handshakes, and ensuring bidirectional communication
//!
//! 4. **Initialization Data**: Constructing comprehensive initialization
//!    payloads for both the Sky frontend and Cocoon extension host
//!
//! ## Architecture
//!
//! The module is divided into two main components:
//!
//! - **CocoonManagement**: Handles the VS Code extension host process (Cocoon)
//!   which provides compatibility with VS Code extensions
//!
//! - **InitializationData**: Constructs and validates initialization payloads
//!   containing workspace information, extension manifests, and system
//!   configuration
//!
//! ## Port Allocation
//!
//! The module reserves specific ports for gRPC communication:
//! - 50051: Mountain Vine server (main process)
//! - 50052: Cocoon Vine server (extension host)
//!
//! ## Feature Flags
//!
//! - `ExtensionHostCocoon`: Enables Cocoon extension host (required for VS Code
//!   extension compatibility)
//!
//! ## Error Handling
//!
//! All operations return `Result<(), CommonError>` with detailed error context
//! for graceful degradation when sidecar processes are unavailable.
//!
//! # Module Contents
//!
//! - [`CocoonManagement`]: Cocoon sidecar process lifecycle management
//! - [`InitializationData`]: Initialization data construction

#![allow(non_snake_case)]

pub mod CocoonManagement;
pub mod ExtractDevTag;
pub mod InitializationData;
pub mod NodeResolver;
