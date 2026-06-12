//! Resolve the Node.js binary used to spawn Cocoon.
//!
//! Ladder (first hit wins, cached in `OnceLock`):
//!   `Pick` override → shipped (`Resources/Node/bin/node`) →
//!   fnm → volta → asdf → nvm → homebrew → PATH `node`.
//!
//! Each step logs its outcome so the resolved source is visible in the log.

/// Checkminmajor module.
pub mod CheckMinMajor;

/// Expandhome module.
pub mod ExpandHome;

/// Nodeexecutablename module.
pub mod NodeExecutableName;

/// Nodesource module.
pub mod NodeSource;

/// Querynodeversion module.
pub mod QueryNodeVersion;

/// Resolvenodebinary module.
pub mod ResolveNodeBinary;

/// Resolveuncached module.
pub mod ResolveUncached;

/// Resolvednode module.
pub mod ResolvedNode;

/// Tryasdf module.
pub mod TryAsdf;

/// Tryfnm module.
pub mod TryFnm;

/// Tryhomebrew module.
pub mod TryHomebrew;

/// Trynvm module.
pub mod TryNvm;

/// Tryoverride module.
pub mod TryOverride;

/// Tryshipped module.
pub mod TryShipped;

/// Tryvolta module.
pub mod TryVolta;
