// @module track
// @description This module acts as the central request dispatcher for the
// Mountain application. It is the primary entry point for all incoming commands
// and RPC calls, whether they originate from the Sky frontend or a Cocoon
// sidecar.
//
// Its main responsibility is to "track" a request to its final destination,
// which is typically by creating a declarative `ActionEffect` to be executed
// by the `ApplicationRunTime`.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---

// Contains the logic for creating `ActionEffect`s from request payloads.
mod EffectCreation;
// Contains the main dispatch functions.
mod TrackLogic;

// --- Public Re-exports ---

// The main dispatch functions for handling requests from both the frontend
// and sidecar processes.
// @see TrackLogic
pub use self::TrackLogic::{DispatchCommand, DispatchSidecarRequest};
