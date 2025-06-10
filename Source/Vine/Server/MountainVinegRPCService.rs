use std::sync::Arc;

use log::{error, info, trace};
use serde_json::Value;
use tauri::{ApplicationHandle, Manager, Wry};
use tonic::{Request, Response, Status};

// @module MountainVinegRPCService
// @description Defines the gRPC service implementation for Mountain. This
// struct handles incoming RPC calls from the Cocoon sidecar, dispatches them
// to the appropriate Handler via the `track` module, and returns the results.
use crate::{Handler::extension_status, RunTime::ApplicationRunTime::ApplicationRunTime, track, vine::generated::*};

pub struct MountainVinegRPCService {
	ApplicationHandle:ApplicationHandle<Wry>,
	RunTime:Arc<ApplicationRunTime>,
}

impl MountainVinegRPCService {
	pub fn New(ApplicationHandle:ApplicationHandle<Wry>, RunTime:Arc<ApplicationRunTime>) -> Self {
		info!("[MountainVinegRPCService] New instance created.");
		Self { ApplicationHandle, RunTime }
	}
}

#[tonic::async_trait]
impl MountainService for MountainVinegRPCService {
	// Handles generic request-response RPCs from Cocoon.
	async fn ProcessCocoonRequest(&self, Request:Request<GenericRequest>) -> Result<Response<GenericResponse>, Status> {
		let RequestData = Request.into_inner();
		let MethodName = RequestData.method;
		let RequestId = RequestData.request_id;

		info!(
			"[VineServer] Received gRPC Request [ID: {}]: Method='{}'",
			RequestId, MethodName
		);

		let ParametersValue = match serde_json::from_slice(&RequestData.params) {
			Ok(v) => v,
			Err(e) => {
				let msg = format!("Failed to deserialize parameters for method '{}': {}", MethodName, e);
				error!("{}", msg);
				return Err(Status::new(tonic::Code::InvalidArgument, msg));
			},
		};
		trace!("[VineServer] Params for [ID: {}]: {:?}", RequestId, ParametersValue);

		let DispatchResult = track::DispatchSidecarRequest(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			"cocoon-main".to_string(), // In the future, this could come from metadata.
			MethodName.clone(),
			ParametersValue,
		)
		.await;

		match DispatchResult {
			Ok(SuccessfulResult) => {
				let ResultBytes = serde_json::to_vec(&SuccessfulResult).unwrap_or_default();
				let ResponsePayload = GenericResponse { request_id:RequestId, result:ResultBytes, error:None };
				Ok(Response::new(ResponsePayload))
			},
			Err(ErrorString) => {
				let ErrorPayload = RpcError { message:ErrorString, code:-32000, data:vec![] };
				let ResponsePayload = GenericResponse { request_id:RequestId, result:vec![], error:Some(ErrorPayload) };
				Ok(Response::new(ResponsePayload))
			},
		}
	}

	// Handles generic fire-and-forget notifications from Cocoon.
	async fn SendCocoonNotification(&self, Request:Request<GenericNotification>) -> Result<Response<Empty>, Status> {
		let NotificationData = Request.into_inner();
		let MethodName = NotificationData.method;

		info!("[VineServer] Received gRPC Notification: Method='{}'", MethodName);

		let ParametersValue = match serde_json::from_slice(&NotificationData.params) {
			Ok(v) => v,
			Err(e) => {
				let msg = format!("Failed to deserialize parameters for notification '{}': {}", MethodName, e);
				error!("{}", msg);
				// For notifications, we don't return an error, just log it and succeed.
				return Ok(Response::new(Empty {}));
			},
		};
		trace!("[VineServer] Notification Params: {:?}", ParametersValue);

		// We can use a special handler for notifications like extension status updates.
		// A more general approach would be to use the main track dispatcher.
		let _ = extension_status::HandleExtensionHostStatusNotification(
			&self.ApplicationHandle,
			&MethodName,
			ParametersValue,
		)
		.await;

		Ok(Response::new(Empty {}))
	}

	// Additional RPC methods from the .proto file would be implemented here.
	// ...
}
