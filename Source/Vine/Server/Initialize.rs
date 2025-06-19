//! # Initialize (Vine Server)
//!
//! Contains the logic to initialize and start the Mountain gRPC server.

use std::{net::SocketAddr, sync::Arc};

use log::{error, info};
use tauri::{AppHandle, Manager};
use tonic::transport::Server;

use super::MountainVinegRPCService::MountainVinegRPCService;
use crate::{RunTime::ApplicationRunTime, Vine::Generated::mountain_service_server::MountainServiceServer};

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
pub fn Initialize(ApplicationHandle:AppHandle, AddressString:String) {
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

		let RunTime = match ApplicationHandle.try_state::<Arc<ApplicationRunTime>>() {
			Some(rt) => rt.inner().clone(),
			None => {
				error!("[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.");
				return;
			},
		};

		let MountainService = MountainVinegRPCService::Create(ApplicationHandle.clone(), RunTime);

		if let Err(e) = Server::builder()
			.add_service(MountainServiceServer::new(MountainService))
			.serve(Address)
			.await
		{
			error!("[VineServer] gRPC server failed to run: {}", e);
		}
	});
}
