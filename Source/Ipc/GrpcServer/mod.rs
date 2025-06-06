
// Declares and exports modules related to the gRPC server,
// which runs within Cocoon to receive requests from the Mountain backend.

#![allow(non_snake_case, non_camel_case_types)]

// Sub-modules for different server-side gRPC request handlers.
mod HandleCancel; // Logic for handling cancellation requests.
mod HandleNotification; // Logic for handling fire-and-forget notifications.
mod HandleRequest; // Logic for handling request/response calls.
mod HandleRpcData; // Logic for handling raw binary RPC data.
mod Initialize; // Handles the initialization and lifecycle of the gRPC server.
mod Server; // Defines the main gRPC service struct and its implementation.

// Re-exporting the primary public functions and types from each module.
pub use self::{
	Initialize::{Get, Initialize, IsRunning, Shutdown},
	Server::CocoonVineGrpcService,
};
