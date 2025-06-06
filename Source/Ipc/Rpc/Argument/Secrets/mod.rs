// File: Ipc/Rpc/Argument/Secrets/mod.rs
// This module defines and exports the argument structures (DTOs) used for
// RPC calls related to secret management (e.g., passwords, API keys).

#![allow(non_snake_case, non_camel_case_types)]

mod GetSecretArgument;
mod SetSecretArgument;

pub use GetSecretArgument::GetSecretArgument;
pub use SetSecretArgument::SetSecretArgument;
