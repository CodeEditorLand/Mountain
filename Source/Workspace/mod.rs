// File: Mountain/Source/Workspace/mod.rs
// Role: Public module interface for workspace-related logic.
// Responsibilities:
//   - Expose services related to managing workspaces, such as parsing
//     `.code-workspace` files.

//! # Workspace Module
//!
//! ## RESPONSIBILITY
//! - Workspace folder management and state tracking
//! - `.code-workspace` file parsing and serialization
//! - Multi-root workspace support
//! - Workspace configuration and settings management
//! - Workspace folder add/remove/reorder operations
//! - Workspace trust and security validation
//!
//! ## ARCHITECTURAL ROLE
//! - Provides workspace services to providers and commands
//! - Integrates with ApplicationState for workspace persistence
//! - Supplies workspace data to Wind/Sky for UI rendering
//! - Handles workspace-related IPC commands from Wind
//!
//! ## DESIGN PATTERNS (Borrowed from VSCode)
//! - Workspace Service (vs/platform/workspace/)
//! - Workspace Configuration pattern
//! - Multi-root workspace management
//! - Workspace folders data structure
//!
//! ## TODO
//! - Implement workspace trust management
//! - Add workspace folder validation
//! - Support workspace templates
//! - Implement workspace sharing and collaboration
//! - Add workspace statistics and metrics
//! - Support workspace snapshots and restore
//! - Implement workspace local history
//! - Add workspace migration and upgrade paths

pub mod WorkspaceFileService;
