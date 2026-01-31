//! # Extension Management Module
//!
//! ## RESPONSIBILITY
//! - Extension discovery and scanning from multiple sources
//! - Extension manifest (package.json) parsing and validation
//! - Extension activation event registration and handling
//! - Extension compatibility checking and version resolution
//! - Extension dependency management and resolution
//!
//! ## ARCHITECTURAL ROLE
//! - Provides extension data to ApplicationState for consumption by Wind/Sky
//! - Integrates with Cocoon for extension activation and lifecycle
//! - Scans extension directories and VS Code marketplace
//! - Validates extension compatibility with editor API version
//!
//! ## DESIGN PATTERNS (Borrowed from VSCode)
//! - Extension Scanner pattern (vs/platform/extensionManagement/)
//! - Manifest validation with schema enforcement
//! - Semantic versioning for compatibility checking
//! - Extension activation events (onStartup, onLanguage, onCommand, etc.)
//!
//! ## TODO
//! - Implement marketplace API integration
//! - Add extension search and filtering
//! - Implement extension dependency graph resolution
//! - Add extension deactivation lifecycle
//! - Implement extension storage quota tracking
//! - Add extension update checking and notifications
//! - Support extension capabilities declarations
//! - Implement extension trust and permission system

#![allow(non_snake_case)]

pub mod Scanner;
