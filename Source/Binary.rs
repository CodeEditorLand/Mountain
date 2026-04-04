#![allow(non_snake_case)]

//! Binary entry point for Mountain.

use Mountain::Binary;

/// Main entry point for both mobile and desktop builds.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Main::Main(); }
