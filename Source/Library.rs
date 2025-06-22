//! # Mountain Crate Library
//!
//! This file conceptually represents the library root for the Mountain
//! application, declaring all of its major internal components. This allows
//! the `Binary.rs` file to have a clean entry point that orchestrates these
//! components.

#![allow(non_snake_case, non_camel_case_types)]
#![feature(trivial_bounds)]

pub mod ApplicationState;

pub mod Command;

pub mod Environment;

pub mod ExtensionManagement;

pub mod FileSystem;

pub mod ProcessManagement;

pub mod RunTime;

pub mod Track;

pub mod Vine;

pub mod WorkSpace;

// The main binary entry point is defined in its own module.
pub mod Binary;

/// The main entry point for mobile builds, which is required by Tauri but
/// delegates to the primary binary logic.
#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Fn(); }
