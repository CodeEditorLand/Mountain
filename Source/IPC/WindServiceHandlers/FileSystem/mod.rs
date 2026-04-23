#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! FileSystem atoms - two tiers:
//! - `Managed`: legacy handlers that route through
//!   `FileSystemReader`/`FileSystemWriter` traits on the Application
//!   runtime. Currently only the binary read/write variants are wired into
//!   dispatch; the rest are preserved for future reuse.
//! - `Native`: URI-aware direct `tokio::fs` handlers that Wind/Sky call via
//!   `file:*` channels.
//!
//! No `pub use` - every call site qualifies through `Managed::<Atom>` or
//! `Native::<Atom>`.

pub mod Managed;
pub mod Native;
