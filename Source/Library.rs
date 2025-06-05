// ---------------------------------------------------------------------------------------------
// Mountain Crate Library Root (lib.rs)
// --------------------------------------------------------------------------------------------
// This file declares all the public modules that constitute the `mountain`
// library. It serves as the entry point for the Rust compiler to understand the
// crate's structure.
// --------------------------------------------------------------------------------------------

// These attributes are typically more relevant for a binary's main.rs rather
// than a lib.rs. Hides the console window on Windows in release builds if this
// lib were compiled as a binary.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Allows non_snake_case identifiers globally for this crate.
#![allow(non_snake_case)]

// Public module declarations:

pub mod app_state;
pub mod environment;
pub mod rpc;
pub mod runtime;
pub mod track;
// pub mod vine; // Assuming vine is a module for IPC logic
// pub mod mist; // Assuming mist is a module, perhaps for WebSocket/Mistral
// related logic pub mod Entry; // If there's an `Entry.rs` file or
// `Entry/mod.rs`

// The `Binary` module and the `main` function below suggest that the
// actual application entry point might be structured within `src/Binary/Fn.rs`
// and `src/main.rs` (the binary crate root) would simply call
// `mountain::main()`. If `mountain` is purely a library, this `main` function
// might not be here.
pub mod Binary {
	// This structure implies that `src/Binary/Fn.rs` (or `src/Binary/Fn/mod.rs`)
	// contains the actual main logic, which we synthesized as `main.rs` previously.
	// If that's the case, the `main.rs` content would live there.
	pub mod Fn {
		// Placeholder if the main logic is indeed in mountain::Binary::Fn::Fn()
		// For our synthesis, the content of `main.rs` (the Tauri entry point)
		// would effectively be what's called by this.
		pub fn Fn() {
			// This would call the actual Tauri application startup logic.
			// If the `main.rs` we synthesized earlier is intended to be called from here,
			// that main function would need to be moved into this module structure
			// or this `Fn()` would need to call it.
			// For now, we assume the synthesized `main.rs` is the direct binary entry.
			panic!("mountain::Binary::Fn::Fn() called. If this is the intended entry, move main logic here.");
		}
	}
}

// If this `main` function is intended for the `mountain` library to provide an
// entry point (e.g., if `mountain` itself is a binary crate, or for testing),
// it would call the structured main logic.
#[allow(dead_code)] // Allow if not used, or if `src/main.rs` is the true entry.
#[cfg_attr(mobile, tauri::mobile_entry_point)] // For Tauri mobile builds.
fn main() {
	// This indirection implies that the actual `main` logic is within
	// `Binary::Fn::Fn()`. If the `main.rs` we synthesized earlier is the direct
	// entry point of a `mountain` binary, then this `main()` function in `lib.rs`
	// might be redundant or for a different purpose.
	Binary::Fn::Fn();
}

pub mod handlers {
	pub mod commands;
	pub mod config;
	pub mod diagnostics;
	pub mod documents;
	// pub mod enablement; // If exists
	pub mod error_utils;
	pub mod extension_status;
	// pub mod native_fs; // If exists
	pub mod output;
	// pub mod process_mgmt; // If exists
	// pub mod protocol; // If exists
	// pub mod proxy; // If exists
	// pub mod registry; // If exists
	pub mod secrets;
	pub mod sky_ui_responses;
	pub mod storage;
	pub mod terminal;
	// pub mod ui; // If exists (distinct from sky_ui_responses)
	// pub mod workspace_fs_api; // If exists
	pub mod workspace;

	// Added from snippets
	pub mod sky_configuration;
	pub mod sky_dtos;
	pub mod sky_ipc_bridge;
	// pub mod sky_commands; // If sky_commands.rs exists in handlers/
}

// If sky_commands is a top-level module rather than under handlers:
// pub mod sky_commands;

// Note: The actual existence and content of modules like `enablement`,
// `native_fs`, `process_mgmt`, `protocol`, `proxy`, `registry`, `ui`,
// `workspace_fs_api`, `vine`, `mist`, `Entry`, `Binary`, and `sky_commands` (if
// top-level) depend on the project's file structure (`src/*.rs` or
// `src/*/mod.rs`). This `lib.rs` declares them; they need to have corresponding
// files.
