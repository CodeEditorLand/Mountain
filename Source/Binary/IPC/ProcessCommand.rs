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
//! - `ProcessGetExecPath::Fn` - current executable path.
//! - `ProcessGetPlatform::Fn` - `darwin` / `win32` / `linux`.
//! - `ProcessGetArch::Fn` - CPU architecture.
//! - `ProcessGetPid::Fn` - running PID.
//! - `ProcessGetShellEnv::Fn` - full environment map.
//! - `ProcessGetMemoryInfo::Fn` - per-platform `{ private,
//!   shared, residentSet }` triple.

pub mod ProcessGetArch;

pub mod ProcessGetExecPath;

pub mod ProcessGetMemoryInfo;

pub mod ProcessGetPid;

pub mod ProcessGetPlatform;

pub mod ProcessGetShellEnv;
