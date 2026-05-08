#![allow(non_snake_case)]

//! # IPC Message Types
//!
//! Core message structures used by every IPC channel between Wind and
//! Mountain.
//!
//! Layout (one export per file, file name = identity):
//! - `IPCMessage::Struct` - generic envelope with id, command, payload,
//!   timestamp, correlation id, priority.
//! - `IPCCommand::Struct` - command request with args and named params.
//! - `IPCResponse::Struct` - success/error response keyed by correlation id.
//! - `MessagePriority::Enum` - Low / Normal / High / Critical.
//!
//! TODO: zero callers as of 2026-05-02. Used to be the planned shape
//! for the Tauri-IPC envelope; superseded by the gRPC channel and
//! direct Tauri events. Kept atomic so a future router can adopt it
//! without re-deriving fields.

pub mod IPCCommand;

pub mod IPCMessage;

pub mod IPCResponse;

pub mod MessagePriority;
