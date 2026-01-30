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
	Vine::{Error::VineError, Generated::{mountain_service_server::MountainServiceServer, cocoon_service_server::CocoonServiceServer}},
};

/// Initializes and starts the gRPC servers on background tasks.
///
/// This function retrieves the core `ApplicationRunTime` from Tauri's managed
/// state, instantiates the gRPC service implementations
/// (`MountainVinegRPCService` and `CocoonServiceServer`), and uses `tonic` to 
/// serve them at the specified addresses.
///
/// # Parameters
/// * `ApplicationHandle`: The Tauri application handle.
/// * `MountainAddressString`: The address and port to bind the Mountain server to (e.g.,
///   `"[::1]:50051"`).
/// * `CocoonAddressString`: The address and port to bind the Cocoon server to (e.g.,
///   `"[::1]:50052"`).
///
/// # Returns
/// A `Result` indicating if the server setup was successful. The servers themselves
/// run on separate spawned tasks.
pub fn Initialize(
	ApplicationHandle:AppHandle, 
	MountainAddressString:String,
	CocoonAddressString:String
) -> Result<(), VineError> {
	let MountainAddress:SocketAddr = MountainAddressString.parse()?;
	let CocoonAddress:SocketAddr = CocoonAddressString.parse()?;

	let RunTime = ApplicationHandle
		.try_state::<Arc<ApplicationRunTime>>()
		.ok_or_else(|| {
			let msg = "[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.";

			error!("{}", msg);

			VineError::ClientNotConnected(msg.to_string())
		})?
		.inner()
		.clone();

	let MountainService = MountainVinegRPCService::Create(ApplicationHandle.clone(), RunTime.clone());
	
	// Create CocoonService server
	let CocoonService = CocoonServiceServer::new(
		RunTime.ApplicationState.clone(),
		RunTime.Environment.clone(),
		RunTime.Require(),
	);

	// Spawn Mountain server to run in the background.
	tokio::spawn(async move {
		info!("[VineServer] Starting Mountain gRPC server on {}", MountainAddress);

		if let Err(e) = Server::builder()
			.add_service(MountainServiceServer::new(MountainService))
			.serve(MountainAddress)
			.await
		{
			error!("[VineServer] Mountain gRPC server failed to run: {}", e);
		}
	});
	
	// Spawn Cocoon server to run in the background.
	tokio::spawn(async move {
		info!("[VineServer] Starting Cocoon gRPC server on {}", CocoonAddress);

		if let Err(e) = Server::builder()
			.add_service(CocoonServiceServer::new(CocoonService))
			.serve(CocoonAddress)
			.await
		{
			error!("[VineServer] Cocoon gRPC server failed to run: {}", e);
		}
	});

	Ok(())
}
