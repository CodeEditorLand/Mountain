//! Mountain-side compat surface for the Air gRPC service provider. The
//! canonical struct and per-method `impl` blocks live in
//! `Element/Air/Source/Client/AirServiceProvider/`. This file re-exposes
//! only what Mountain code still references by path:
//!
//! - [`AirServiceProvider`] type alias to the canonical type.
//! - [`GenerateRequestID`] submodule - thin delegator to the canonical
//!   helper.

pub mod GenerateRequestID;

/// High-level provider over [`super::AirClient::AirClient`]. The
/// canonical definition lives in
/// `::AirLibrary::Client::AirServiceProvider::AirServiceProvider`; every
/// per-method `impl` block, the constructor, and accessors are owned by
/// the Air crate.
pub type AirServiceProvider = ::AirLibrary::Client::AirServiceProvider::AirServiceProvider;
