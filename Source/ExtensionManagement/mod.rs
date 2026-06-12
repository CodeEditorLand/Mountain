//! Extension lifecycle: scan bundled and user-installed extension trees, parse
//! their `package.json` manifests, install VSIX archives. Mountain owns the
//! discovery surface; activation runs in Cocoon over gRPC.
//!
//! ## Sub-modules
//!
//! - [`Scanner`]: Extension directory scanning and manifest parsing
//! - [`VsixInstaller`]: VSIX extension archive installation

/// Extension directory scanner: discovers bundled and user-installed
/// extensions.
pub mod Scanner;

/// VSIX extension archive installation and extraction.
pub mod VsixInstaller;
