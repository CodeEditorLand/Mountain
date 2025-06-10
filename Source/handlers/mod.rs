

/**
 * @module handlers
 * @description This is the main aggregator for the entire business logic layer of
 * the Mountain application.
 *
 * Each sub-module contains the concrete implementation logic for a specific
 * feature or service contract defined in the `Common` crate. The `environment`
 * providers delegate their work to the functions within these modules.
 */

#![allow(non_snake_case, non_camel_case_types)]

// --- Core Service Handlers (alphabetical) ---
pub mod commands;
pub mod config;
pub mod custom_editor;
pub mod diagnostics;
pub mod documents;
pub mod extension_management;
pub mod extension_status;
pub mod fs;
pub mod language_features;
pub mod output;
pub mod secrets;
pub mod status_bar;
pub mod storage;
pub mod terminal;
pub mod tree_view;
pub mod ui;
pub mod webview;
pub mod workspace;

// --- Internal and Bridge Handlers ---
pub mod error_utils;
pub mod process_management;
pub mod protocol;
pub mod sky_commands;
pub mod sky_ipc_bridge;
pub mod sky_ui_responses;
