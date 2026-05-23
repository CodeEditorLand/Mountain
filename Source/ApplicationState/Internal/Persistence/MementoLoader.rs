
//! # MementoLoader - Persistence layer
//!
//! Loads `ApplicationState` memento JSON from disk during boot.
//! Two flavours: best-effort (returns empty on failure) and
//! result-typed (surfaces failures explicitly during recovery).
//!
//! Layout (one export per file, file name = identity):
//! - `LoadInitialMementoFromDisk::Fn` - best-effort loader for
//!   `ApplicationState::default()`. Backs up corrupted files, creates the
//!   parent directory on read errors.
//! - `LoadMementoWithRecovery::Fn` - result-typed loader used during recovery
//!   flows; surfaces FS / parse failures.
//! - `AttemptMementoRecovery::Fn` (internal) - write a `.backup` sibling for
//!   the corrupted content.
//! - `CreateCorruptedBackup::Fn` (internal) - write a timestamped
//!   `.json.corrupted.<ts>` sibling.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02 - pending wire-up from
//! `Environment::StorageProvider` boot path.

pub mod AttemptMementoRecovery;

pub mod CreateCorruptedBackup;

pub mod LoadInitialMementoFromDisk;

pub mod LoadMementoWithRecovery;
