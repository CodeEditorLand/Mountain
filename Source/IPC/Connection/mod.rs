//! Connection lifecycle, pooling, and health monitoring for IPC. Submodules:
//! `Manager` (pool + handles), `Types` (`ConnectionHandle`, `Stats`),
//! `Health` (background checker). Callers spell the full path; no `pub use`.

pub mod Health;

pub mod Manager;

pub mod Types;
