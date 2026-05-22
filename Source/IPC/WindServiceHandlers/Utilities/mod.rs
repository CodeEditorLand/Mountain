#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Utilities for Wind handlers, grouped by purpose. Sub-module helpers
//! (`hex_digit`, `percent_decode`, `normalize_uri_path`, etc.) stay co-
//! located with their single public entry point - splitting by strict one-
//! fn-per-file would fragment tightly coupled internals for no readability
//! gain.
//!
//! No `pub use`. External callers must spell
//! `Utilities::<Domain>::<Function>`.

pub mod ApplicationRoot;

pub mod ChannelPriority;

pub mod LocalhostUrl;

pub mod FiddeeRoot;

pub mod JsonValueHelpers;

pub mod MetadataEncoding;

pub mod PathExtraction;

pub mod RecentlyOpened;

pub mod UserdataDir;
