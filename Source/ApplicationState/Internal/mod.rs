//! # Internal
//!
//! Private utilities for ApplicationState: persistence, path resolution,
//! serialization, extension scanning, text processing, and state recovery.
//! These helpers are not part of the public API.

pub mod ExtensionScanner;

pub mod PathResolution;

pub mod Persistence;

pub mod Recovery;

pub mod Serialization;

pub mod TextProcessing;
