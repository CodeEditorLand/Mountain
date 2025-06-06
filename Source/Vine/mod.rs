// File: Vine/Mod.rs
// This module handles Inter-Process Communication (IPC) between Mountain (Rust
// backend) and Cocoon (Node.js sidecar), specifically using gRPC. It defines
// error types, message structures, and client/server implementations for this
// communication.

// Manual Protobuf-like message definitions, to be replaced or supplemented by
// generated code. These are kept for now as they were part of the "current
// state" and might contain nuances not yet fully captured by the .proto file or
// its generation. Ideally, these would be generated from `vine.proto`.
mod VineGrpcManual;
pub use self::VineGrpcManual::*; // Re-export to maintain current usage patterns.

// Protobuf generated types (conceptual path, actual path might differ based on
// build process). This assumes `tonic-build` or a similar tool generates Rust
// code from `vine.proto`.
pub mod VineGrpcPb {
	// Example: tonic::include_proto!("vine_ipc");
	// For now, as per current state, re-exporting from VineGrpcManual
	// as the .proto and generated code might not be fully integrated yet.
	pub use crate::Vine::VineGrpcManual::*;
}

// gRPC client-side logic for Mountain to call Cocoon.
mod Client;
pub use Client::{
	ConnectToCocoonService as Connect,
	SendCancelToCocoonService as SendCancel,
	SendNotificationToCocoonService as SendNotification,
	SendRequestToCocoonService as SendRequest,
	SendRpcDataToCocoonService as SendRpcData,
	UnregisterCocoonClient as Unregister,
};

// gRPC server-side logic for Mountain to receive calls from Cocoon.
pub mod Server {
	mod MountainVineGrpcService; // Renamed from MountainVineGrpcService
	pub use self::MountainVineGrpcService::*;
}

// Vine-specific error types.
mod VineError;
pub use self::VineError::VineError;

// Legacy Vine message structures (from stdio IPC), potentially for reference or
// gradual phasing out. If gRPC is fully adopted, these might become obsolete.
mod VineMessage; // Renamed from Vinemessage
pub use self::VineMessage::{VineMessage, VineMessageType};
