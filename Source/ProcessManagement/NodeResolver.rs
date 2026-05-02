#![allow(non_snake_case, dead_code)]

//! Resolve the Node.js binary used to spawn Cocoon.
//!
//! Ladder (first hit wins, cached in `OnceLock`):
//!   `Pick` override → shipped (`Resources/Node/bin/node`) →
//!   fnm → volta → asdf → nvm → homebrew → PATH `node`.
//!
//! Each step logs its outcome so the resolved source is visible in the log.

pub mod CheckMinMajor;
pub mod ExpandHome;
pub mod NodeExecutableName;
pub mod NodeSource;
pub mod QueryNodeVersion;
pub mod ResolveNodeBinary;
pub mod ResolveUncached;
pub mod ResolvedNode;
pub mod TryAsdf;
pub mod TryFnm;
pub mod TryHomebrew;
pub mod TryNvm;
pub mod TryOverride;
pub mod TryShipped;
pub mod TryVolta;
