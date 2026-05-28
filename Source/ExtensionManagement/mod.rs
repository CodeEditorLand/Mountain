//! Extension lifecycle: scan bundled + user-installed extension trees, parse
//! their `package.json` manifests, install VSIX archives. Mountain owns the
//! discovery surface; activation runs in Cocoon over gRPC.

pub mod Scanner;

pub mod VsixInstaller;
