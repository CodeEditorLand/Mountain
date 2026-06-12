//! Extension lifecycle: scan bundled + user-installed extension trees, parse
//! their `package.json` manifests, install VSIX archives. Mountain owns the
//! discovery surface; activation runs in Cocoon over gRPC.

/// Scanner module.
pub mod Scanner;

/// Vsixinstaller module.
pub mod VsixInstaller;
