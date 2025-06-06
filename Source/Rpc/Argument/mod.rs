// File: Rpc/Argument/mod.rs
// This module serves as a container for all Data Transfer Object (DTO)
// modules used in RPC communication. Each sub-module typically defines
// the argument structures for a specific set of RPC methods or a service.

pub mod Commands;
pub mod Common;
pub mod Configuration;
pub mod Diagnostics;
pub mod Documents;
pub mod Enablement;
pub mod LanguageFeatures;
pub mod Output;
pub mod Secrets;
pub mod StatusBar; // Renamed from statusbar
pub mod Storage;
pub mod Terminal;
pub mod Window;
pub mod Workspace;
