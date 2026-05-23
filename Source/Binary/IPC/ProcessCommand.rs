
//! # ProcessCommand - Wind ProcessPolyfill bridge
//!
//! Tauri commands invoked directly (not through `MountainIPCInvoke`)
//! by Wind's `ProcessPolyfill`. Each command file holds exactly one
//! `#[tauri::command]` whose function name IS the wire identifier
//! (Naming-Convention exception); the snake_case names are required
//! to mirror Node's `process.*` so renderer code that imports
//! `process` keeps working unchanged.
//!
//! Layout (one Tauri command per file):
//! - `process_get_exec_path::process_get_exec_path` - current executable path.
//! - `process_get_platform::process_get_platform` - `darwin` / `win32` /
//!   `linux`.
//! - `process_get_arch::process_get_arch` - CPU architecture.
//! - `process_get_pid::process_get_pid` - running PID.
//! - `process_get_shell_env::process_get_shell_env` - full environment map.
//! - `process_get_memory_info::process_get_memory_info` - per-platform `{
//!   private, shared, residentSet }` triple.

pub mod process_get_arch;

pub mod process_get_exec_path;

pub mod process_get_memory_info;

pub mod process_get_pid;

pub mod process_get_platform;

pub mod process_get_shell_env;
