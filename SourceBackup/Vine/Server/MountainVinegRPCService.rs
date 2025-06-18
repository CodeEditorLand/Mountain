// @module MountainVinegRPCService
// @description Defines the gRPC service implementation for Mountain. This
// struct handles incoming RPC calls from the Cocoon sidecar, dispatches them
// to the appropriate Handler via the `track` module, and returns the results.

use std::sync::Arc;

use log::{error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tonic::{Request, Response, Status};

use crate::{
	track,
	Vine::generated::{mountain_service_server::MountainService, *},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// The concrete implementation of the `MountainService` gRPC service.
pub struct MountainVinegRPCService<R: Runtime> {
	ApplicationHandle: AppHandle<R>,
	RunTime: Arc<ApplicationRunTime>,
}

impl<R: Runtime> MountainVinegRPCService<R> {
	/// Creates a new instance of the Mountain gRPC service.
	pub fn New(app_handle: AppHandle<R>, run_time: Arc<ApplicationRunTime>) -> Self {
		info!("[MountainVinegRPCService] New instance created.");
		Self { ApplicationHandle: app_handle, RunTime: run_time }
	}
}

#[tonic::async_trait]
impl<R: Runtime + 'static> MountainService for MountainVinegRPCService<R> {
	/// Handles generic request-response RPCs from Cocoon.
	async fn process_cocoon_request(&self, request: Request<GenericRequest>) -> Result<Response<GenericResponse>, Status> {
		let request_data = request.into_inner();
		let method_name = request_data.method;
		let request_id = request_data.request_id;

		info!("[VineServer] Received gRPC Request [ID: {}]: Method='{}'", request_id, method_name);

		let parameters_value: Value = match serde_json::from_slice(&request_data.params) {
			Ok(v) => v,
			Err(e) => {
				let msg = format!("Failed to deserialize parameters for method '{}': {}", method_name, e);
				error!("{}", msg);
				let error_payload = RpcError { message: msg, code: -32700, data: vec![] }; // Parse Error
				let response_payload = GenericResponse { request_id, result: vec![], error: Some(error_payload) };
				return Ok(Response::new(response_payload));
			},
		};
		trace!("[VineServer] Params for [ID: {}]: {:?}", request_id, parameters_value);

		// Use the primary track dispatcher to handle the request. This will attempt
		// to map it to an ActionEffect first.
		let dispatch_result = track::DispatchSidecarRequest(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			"cocoon-main".to_string(), // In the future, this could come from connection metadata.
			method_name.clone(),
			parameters_value,
		)
		.await;

		match dispatch_result {
			Ok(successful_result) => {
				let result_bytes = serde_json::to_vec(&successful_result).unwrap_or_else(|e| {
					error!("Failed to serialize successful result for '{}': {}", method_name, e);
					b"null".to_vec()
				});
				let response_payload = GenericResponse { request_id, result: result_bytes, error: None };
				Ok(Response::new(response_payload))
			},
			Err(error_string) => {
				let error_payload = RpcError { message: error_string, code: -32000, data: vec![] }; // Generic Server Error
				let response_payload = GenericResponse { request_id, result: vec![], error: Some(error_payload) };
				Ok(Response::new(response_payload))
			},
		}
	}

	/// Handles generic fire-and-forget notifications from Cocoon.
	async fn send_cocoon_notification(&self, request: Request<GenericNotification>) -> Result<Response<Empty>, Status> {
		let notification_data = request.into_inner();
		let method_name = notification_data.method;

		info!("[VineServer] Received gRPC Notification: Method='{}'", method_name);

		let parameters_value: Value = match serde_json::from_slice(¬ification_data.params) {
			Ok(v) => v,
			Err(e) => {
				error!("Failed to deserialize parameters for notification '{}': {}. Ignoring.", method_name, e);
				return Ok(Response::new(Empty {}));
			},
		};
		trace!("[VineServer] Notification Params: {:?}", parameters_value);

		// Notifications are often for status updates and don't fit the effect model well.
		// We route them to the direct RPC fallback handler.
		let _ = crate::Handler::rpc::RouteRpcCall(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			"cocoon-main".to_string(),
			method_name,
			parameters_value,
		)
		.await
		.map_err(|e| warn!("[VineServer] Error handling notification: {}", e)); // Log errors but don't fail the RPC

		Ok(Response::new(Empty {}))
	}

	/// Handles a request from Cocoon to cancel a long-running operation.
	async fn cancel_operation(&self, _request: Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		// A real implementation would map the request_id_to_cancel to a cancellation token
		// and trigger it.
		warn!("[VineServer] Received CancelOperation request, but cancellation is not yet implemented.");
		Ok(Response::new(Empty {}))
	}
}
