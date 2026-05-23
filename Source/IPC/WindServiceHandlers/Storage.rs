
//! # Persistent Storage handlers
//!
//! Wind invokes these via the WindServiceHandlers dispatcher; each
//! delegates to `Environment::Require<dyn StorageProvider>`. Two
//! scopes: workspace (`false`) for `StorageGet`/`StorageSet`,
//! global (`true`) for the rest - VS Code's storage service
//! distinguishes the two; we follow.
//!
//! Layout (one export per file, file name = identity):
//! - `StorageGet::StorageGet` - single key read (workspace).
//! - `StorageSet::StorageSet` - single key write (workspace).
//! - `StorageDelete::StorageDelete` - single key delete (global).
//! - `StorageKeys::StorageKeys` - list every key (global).
//! - `StorageGetItems::StorageGetItems` - bulk read as `[key,value]` tuples;
//!   called by VS Code's `NativeWorkbenchStorageService` at boot.
//! - `StorageUpdateItems::StorageUpdateItems` - bulk insert + delete; matches
//!   `IndexedDBStorageDatabase`'s wire shape.

pub mod StorageDelete;

pub mod StorageGet;

pub mod StorageGetItems;

pub mod StorageKeys;

pub mod StorageSet;

pub mod StorageUpdateItems;
