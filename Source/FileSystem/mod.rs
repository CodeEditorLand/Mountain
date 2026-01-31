//! # FileSystem Module
//!
//! ## RESPONSIBILITY
//! - File system abstraction and service providers
//! - File explorer view provider for the sidebar.
//! - Directory tree navigation and state management
//! - File and folder operations (create, rename, delete, copy, move)
//! - Symbolic link detection and handling
//! - File system watching and change notifications
//!
//! ## ARCHITECTURAL ROLE
//! - Provides FileExplorerViewProvider for Environment/TreeViewProvider
//! - Integrates with Common::FileSystem traits for file operations
//! - Supplies file data to Wind/Sky explorer UI
//! - Handles file system events and notifications
//!
//! ## DESIGN PATTERNS (Borrowed from VSCode)
//! - File System Service (vs/platform/files/)
//! - File Explorer View Provider (vs/workbench/contrib/files/)
//! - File System Provider interface pattern
//! - URI-based file identification
//!
//! ## TODO
//! - Implement file system change watching (watchdog)
//! - Add file search and filtering capabilities
//! - Implement virtual file system support
//! - Add file system statistics and metrics
//! - Support for network file systems (SMB, FTP, SFTP)
//! - Implement file thumbnail previews
//! - Add file system quota and space management
//! - Support for compressed file navigation

#![allow(non_snake_case)]

pub mod FileExplorerViewProvider;
