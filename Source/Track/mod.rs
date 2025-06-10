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
// Defines DTOs for commands invoked directly from the Sky frontend.
mod SkyCommandDto;
// Contains the main dispatch functions.
mod TrackLogic;

// --- Public Re-exports ---

// Re-exports all DTOs used for Sky-to-Mountain commands.
// @see SkyCommandDto
//
pub use self::SkyCommandDto::*;
// The main dispatch functions for handling requests from both the frontend
// and sidecar processes.
// @see TrackLogic
pub use self::TrackLogic::{DispatchCommand, DispatchSidecarRequest};
