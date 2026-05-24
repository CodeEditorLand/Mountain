//! `DocumentSyncCommand` - atomized.

pub mod MountainAddDocumentForSync;
pub mod MountainGetSyncStatus;

pub use MountainAddDocumentForSync::Fn as MountainAddDocumentForSync;
pub use MountainGetSyncStatus::Fn as MountainGetSyncStatus;
