//! # CocoonServiceServer
//!
//! Implements the gRPC server for Mountain-Cocoon communication.

use std::sync::Arc;

use log::{debug, error, info};
use tonic::{Request, Response, Status};
use async_trait::async_trait;
use CommonLibrary::{
	Environment::Requires::Requires,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
};

use super::super::Generated::{
	CancelOperationRequest,
	Empty,
	GenericNotification,
	GenericRequest,
	GenericResponse,
	RpcError,
	cocoon_service_server::CocoonService,
};
use crate::{ApplicationState::ApplicationState, Environment::MountainEnvironment::MountainEnvironment};

/// Implementation of the CocoonService gRPC server
pub struct CocoonServiceImpl {
	/// Mountain environment
	environment:Arc<MountainEnvironment>,
}

impl CocoonServiceImpl {
	/// Creates a new instance of the CocoonService server
	pub fn new(environment:Arc<MountainEnvironment>) -> Self {
		info!("[CocoonServiceImpl] New instance created");

		Self { environment }
	}

	/// Handle generic Mountain requests from Cocoon
	async fn handle_mountain_request(&self, request:GenericRequest) -> Result<GenericResponse, Status> {
		debug!(
			"[CocoonServiceImpl] Handling Mountain request '{}' with ID {}",
			request.method, request.request_identifier
		);

		match request.method.as_str() {
			"InitializeExtensionHost" => {
				info!("[CocoonServiceImpl] Initializing extension host");

				// Return success response
				Ok(GenericResponse {
					request_identifier:request.request_identifier,
					result:serde_json::to_vec(&"initialized")
						.map_err(|e| Status::internal(format!("Failed to serialize response: {}", e)))?,
					error:None,
				})
			},

			"GetExtensions" => {
				debug!("[CocoonServiceServer] Getting extensions");

				let extension_service:Arc<dyn ExtensionManagementService> = (*self.environment).Require();
				let extensions = extension_service
					.GetExtensions()
					.await
					.map_err(|e| Status::internal(format!("Failed to get extensions: {}", e)))?;

				Ok(GenericResponse {
					request_identifier:request.request_identifier,
					result:serde_json::to_vec(&extensions)
						.map_err(|e| Status::internal(format!("Failed to serialize extensions: {}", e)))?,
					error:None,
				})
			},

			"ActivateExtension" => {
				debug!("[CocoonServiceServer] ActivateExtension not implemented");
				Err(Status::unimplemented("ActivateExtension method not available"))
			},

			_ => {
				error!("[CocoonServiceServer] Unknown Mountain request method '{}'", request.method);

				Err(Status::unimplemented(format!("Unknown method: {}", request.method)))
			},
		}
	}
}

#[async_trait]
impl CocoonService for CocoonServiceImpl {
	/// Process Mountain requests from Cocoon
	async fn process_mountain_request(
		&self,
		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let request_data = request.into_inner();

		match self.handle_mountain_request(request_data).await {
			Ok(response) => Ok(Response::new(response)),
			Err(status) => {
				error!("[CocoonServiceImpl] Error processing Mountain request: {}", status);
				Err(status)
			},
		}
	}

	/// Send Mountain notifications to Cocoon
	async fn send_mountain_notification(
		&self,
		request:Request<GenericNotification>,
	) -> Result<Response<Empty>, Status> {
		let notification = request.into_inner();

		debug!("[CocoonServiceServer] Received Mountain notification '{}'", notification.method);

		// Handle notifications (fire-and-forget)
		match notification.method.as_str() {
			"ExtensionActivated" => {
				info!("[CocoonServiceServer] Extension activated notification");
			},

			"ExtensionDeactivated" => {
				info!("[CocoonServiceServer] Extension deactivated notification");
			},

			_ => {
				debug!("[CocoonServiceServer] Unknown Mountain notification '{}'", notification.method);
			},
		}

		Ok(Response::new(Empty {}))
	}

	/// Cancel operations requested by Mountain
	async fn cancel_operation(&self, request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		let cancel_request = request.into_inner();

		info!(
			"[CocoonServiceServer] Cancelling operation with ID {}",
			cancel_request.request_identifier_to_cancel
		);

		// TODO: Implement operation cancellation logic

		Ok(Response::new(Empty {}))
	}
}
