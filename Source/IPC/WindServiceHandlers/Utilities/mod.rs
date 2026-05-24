//! Utilities for Wind handlers - one `pub fn Fn` per atomic file.
//! Shared-state modules (ApplicationRoot, LocalhostUrl, UserdataDir,
//! RecentlyOpened) are directory modules; `#[path]` overrides are required
//! because the old .rs files still exist in the same directory.

#[path = "ApplicationRoot/mod.rs"]
pub mod ApplicationRoot;

pub mod ChannelPriority;

pub mod FiddeeRoot;

#[path = "JsonValueHelpers/mod.rs"]
pub mod JsonValueHelpers;

#[path = "LocalhostUrl/mod.rs"]
pub mod LocalhostUrl;

pub mod MetadataEncoding;

pub mod PathExtraction;

pub mod PercentDecode;

#[path = "RecentlyOpened/mod.rs"]
pub mod RecentlyOpened;

#[path = "UserdataDir/mod.rs"]
pub mod UserdataDir;
