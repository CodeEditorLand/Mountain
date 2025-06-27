//! # Initialize (Vine Server)
//!
//! Contains the logic to initialize and start the Mountain gRPC server.

#![allow(non_snake_case, non_camel_case_types)]

use std::{net::SocketAddr, sync::Arc};

use log::{error, info};
use tauri::{AppHandle, Manager};
use tonic::transport::Server;

use super::MountainVinegRPCService::MountainVinegRPCService;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine::{Error::VineError, Generated::mountain_service_server::MountainServiceServer},
};

/// Initializes and starts the gRPC server on a background task.
///
/// This function retrieves the core `ApplicationRunTime` from Tauri's managed
/// state, instantiates the gRPC service implementation
/// (`MountainVinegRPCService`), and uses `tonic` to serve it at the specified
/// address.
///
/// # Parameters
/// * `ApplicationHandle`: The Tauri application handle.
/// * `AddressString`: The address and port to bind the server to (e.g.,
///   "[::1]:50051").
///
/// # Returns
/// A `Result` indicating if the server setup was successful. The server itself
/// runs on a separate spawned task.
pub fn Initialize(ApplicationHandle:AppHandle, AddressString:String) -> Result<(), VineError> {
	let Address:SocketAddr = AddressString.parse()?;

	let RunTime = ApplicationHandle
		.try_state::<Arc<ApplicationRunTime>>()
		.ok_or_else(|| {
			let msg = "[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.";

			error!("{}", msg);

			VineError::ClientNotConnected(msg.to_string())
		})?
		.inner()
		.clone();

	let MountainService = MountainVinegRPCService::Create(ApplicationHandle.clone(), RunTime);

	// Spawn the server to run in the background.
	tokio::spawn(async move {
		info!("[VineServer] Starting gRPC server on {}", Address);

		if let Err(e) = Server::builder()
			.add_service(MountainServiceServer::new(MountainService))
			.serve(Address)
			.await
		{
			error!("[VineServer] gRPC server failed to run: {}", e);
		}
	});

	Ok(())
}
