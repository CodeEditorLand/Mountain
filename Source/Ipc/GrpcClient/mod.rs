// File: Ipc/GrpcClient/mod.rs
// Declares and exports modules related to the gRPC client,
// which is used by Cocoon to communicate with the Mountain backend.

#![allow(non_snake_case, non_camel_case_types)]

// Sub-modules for different client-side gRPC operations.
mod Client; // Defines the main gRPC client struct and its connection logic.
mod Initialize; // Handles the initialization and singleton management of the client.
mod SendCancel; // Logic for sending a cancellation request.
mod SendNotification; // Logic for sending a fire-and-forget notification.
mod SendRequest; // Logic for sending a request and awaiting a response.
mod SendRpcData; // Logic for sending raw binary RPC data.

// Re-exporting the primary public functions from each module for easy access.
pub use self::{
	Client::CocoonMountainGrpcClient,
	Initialize::{CloseConnection as Close, GetClientInstance as Get, Initialize, IsConnected},
	SendCancel::SendCancelToMountain as SendCancel,
	SendNotification::SendNotificationToMountain as SendNotification,
	SendRequest::SendRequestToMountain as SendRequest,
	SendRpcData::SendRpcDataToMountain as SendRpcData,
};
