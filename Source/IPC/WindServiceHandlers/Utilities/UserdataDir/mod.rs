#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Canonical userdata base directory (Tauri `app_data_dir`) + first-access
//! scaffolding. State held here; atomics expose Get/Set/Ensure.

pub(crate) static BASE_DIR:std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub(crate) static INITIALIZED:std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub mod Ensure;

pub mod Get;

pub mod Set;
