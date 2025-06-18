// @module Initialize (Vine/server)
// @description Contains the logic to initialize and start the Mountain gRPC
// server.

use std::{net::SocketAddr, sync::Arc};

use log::{error, info, warn};
use tauri::{AppHandle, Manager, Runtime};
use tonic::transport::Server;

use super::MountainVinegRPCService::MountainVinegRPCService;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine::generated::mountain_service_server::MountainServiceServer,
};

/// Initializes and starts the gRPC server on a background task.
///
/// This function retrieves the core `ApplicationRunTime` from Tauri's managed
/// state, instantiates the gRPC service implementation
/// (`MountainVinegRPCService`), and uses `tonic` to serve it at the specified
/// address.
///
/// @param app_handle - The Tauri application handle.
/// @param address_string - The address and port to bind the server to (e.g.,
/// "[::1]:50051").
pub fn Initialize<R:Runtime>(app_handle:AppHandle<R>, address_string:String) {
	tokio::spawn(async move {
		let address:SocketAddr = match address_string.parse() {
			Ok(addr) => addr,
			Err(e) => {
				error!(
					"[VineServer] Invalid gRPC server address '{}': {}. Server will not start.",
					address_string, e
				);
				return;
			},
		};

		info!("[VineServer] Starting gRPC server on {}", address);

		// Retrieve the managed ApplicationRunTime instance. This is the core execution
		// engine.
		let run_time = match app_handle.try_state::<Arc<ApplicationRunTime>>() {
			Some(rt) => rt.inner().clone(),
			None => {
				error!("[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.");
				return;
			},
		};

		// Instantiate our gRPC service implementation.
		let mountain_service = MountainVinegRPCService::New(app_handle.clone(), run_time);

		// Build and run the tonic server.
		let server_result = Server::builder()
			.add_service(MountainServiceServer::new(mountain_service))
			.serve(address)
			.await;

		if let Err(e) = server_result {
			error!("[VineServer] gRPC server failed to run: {}", e);
		} else {
			warn!("[VineServer] gRPC server has shut down unexpectedly.");
		}
	});
}
