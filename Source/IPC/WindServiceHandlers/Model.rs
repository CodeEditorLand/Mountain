#![allow(non_snake_case)]

//! # Text Model registry + TextFile handlers
//!
//! Two related responsibilities sharing the same dispatcher
//! family:
//!
//! - `Model*` - Monaco-side text model registry. Mountain owns
//!   `ApplicationState.Feature.Documents`; Wind reads/writes through these
//!   handlers so the document state survives a webview reload.
//! - `Textfile*` - disk-only paths that bypass the registry. Used for tooling
//!   reads (settings, manifests) and for the actual save round-trip.
//!
//! Layout (one export per file, file name = identity):
//! - `ModelOpen::ModelOpen`, `ModelClose::ModelClose`, `ModelGet::ModelGet`,
//!   `ModelGetAll::ModelGetAll`, `ModelUpdateContent::ModelUpdateContent`.
//! - `TextfileRead::TextfileRead`, `TextfileWrite::TextfileWrite`,
//!   `TextfileSave::TextfileSave`.

pub mod ModelClose;
pub mod ModelGet;
pub mod ModelGetAll;
pub mod ModelOpen;
pub mod ModelUpdateContent;
pub mod TextfileRead;
pub mod TextfileSave;
pub mod TextfileWrite;
