#![allow(non_snake_case)]

//! Extension-related state. Three sub-stores (`ExtensionRegistry`,
//! `ProviderRegistration`, `ScannedExtensions`) plus the composite `State`
//! struct. Callers spell the full sub-path.

pub mod ExtensionRegistry;
pub mod ProviderRegistration;
pub mod ScannedExtensions;
pub mod State;
