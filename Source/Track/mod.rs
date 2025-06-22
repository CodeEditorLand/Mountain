//! # Track Module
//!
//! This module acts as the central request dispatcher for the Mountain
//! application. It is the primary entry point for all incoming commands and RPC
//! calls, whether they originate from the `Sky` frontend or a `Cocoon`
//! sidecar.
//!
//! Its main responsibility is to "track" a request to its final destination by
//! creating a declarative `ActionEffect` that is then executed by the
//! `ApplicationRunTime`.

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---

/// Contains the main dispatch functions.
pub mod DispatchLogic;
/// Contains the logic for creating `ActionEffect`s from request payloads.
pub mod EffectCreation;
