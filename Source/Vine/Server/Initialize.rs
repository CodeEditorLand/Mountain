use std::{net::SocketAddr, sync::Arc};

use log::{error, info, warn};
use tauri::{ApplicationHandle, Manager, Wry};
use tonic::transport::Server;

// @module Initialize (Vine/Server)
// @description Contains the logic to initialize and start the Mountain gRPC
// server.
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use crate::vine::{generated::MountainServiceServer, server::MountainVinegRPCService::MountainVinegRPCService};

// Initializes and starts the gRPC server on a background task.
//
// This function retrieves the core `ApplicationRunTime` from Tauri's managed
// state, instantiates the gRPC service implementation
// (`MountainVinegRPCService`), and uses `tonic` to serve it at the specified
// address.
//
// @param ApplicationHandle - The Tauri application handle.
// @param AddressString - The address and port to bind the server to (e.g.,
// "[::1]:50051").
pub fn Initialize(ApplicationHandle:ApplicationHandle<Wry>, AddressString:String) {
	tokio::spawn(async move {
		let Address:SocketAddr = match AddressString.parse() {
			Ok(addr) => addr,
			Err(e) => {
				error!(
					"[VineServer] Invalid gRPC server address '{}': {}. Server will not start.",
					AddressString, e
				);
				return;
			},
		};

		info!("[VineServer] Starting gRPC server on {}", Address);

		// Retrieve the managed ApplicationRunTime instance. This is the core execution
		// engine.
		let RunTime = match ApplicationHandle.try_state::<Arc<ApplicationRunTime>>() {
			Some(rt) => rt.inner().clone(),
			None => {
				error!("[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.");
				return;
			},
		};

		// Instantiate our gRPC service implementation.
		let MountainService = MountainVinegRPCService::New(ApplicationHandle.clone(), RunTime);

		// Build and run the tonic server.
		let serverResult = Server::builder()
			.add_service(MountainServiceServer::new(MountainService))
			.serve(Address)
			.await;

		if let Err(e) = serverResult {
			error!("[VineServer] gRPC server failed to run: {}", e);
		} else {
			warn!("[VineServer] gRPC server has shut down unexpectedly.");
		}
	});
}
