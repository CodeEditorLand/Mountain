// File: Ipc/Rpc/Argument/mod.rs
// This module serves as a container for all Data Transfer Object (DTO)
// modules used in RPC communication. Each sub-module typically defines
// the argument structures for a specific set of RPC methods or a service.

#![allow(non_snake_case, non_camel_case_types)]

pub mod Commands;
pub mod Common;
pub mod Configuration;
pub mod Diagnostics;
pub mod Documents;
pub mod Enablement;
pub mod LanguageFeatures;
pub mod Output;
pub mod Secrets;
pub mod StatusBar;
pub mod Storage;
pub mod Terminal;
pub mod Window;
pub mod Workspace;
