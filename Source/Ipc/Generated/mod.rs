// File: Ipc/Generated/mod.rs
// Declares and re-exports the Protobuf-generated types for gRPC communication.

#![allow(non_snake_case, non_camel_case_types)]

pub mod VineGrpcPb {
	// This module re-exports the manually adapted, Prost-compatible structs
	// from `VineGrpcManual.rs`. In a build process using `tonic-build`, this
	// would instead contain the `include_proto!` macro to bring in the
	// generated code directly.
	pub use crate::Vine::VineGrpcManual::*;
}
