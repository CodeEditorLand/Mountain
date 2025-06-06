
// This module defines and exports a registry for request handlers.
// Its primary role is to map method names to their corresponding handler
// functions, providing a structured way to manage and dispatch incoming RPC
// calls.

#![allow(non_snake_case, non_camel_case_types)]

mod Registry; // Contains the HandlerRegistry struct and related definitions

pub use self::Registry::*; // Re-export all public items from Registry.rs
