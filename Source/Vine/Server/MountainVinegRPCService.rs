//! # MountainVinegRPCService
//!
//! Defines the gRPC service implementation for Mountain. This struct handles
//! incoming RPC calls from the `Cocoon` sidecar, dispatches them to the
//! application's core logic via the `Track` module, and returns the results.

use std::sync::Arc;

use log::{error, info, trace};
use serde_json::Value;
use tauri::AppHandle;
use tonic::{Request, Response, Status};

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track,
	Vine::Generated::{
		CancelOperationRequest,
		Empty,
		GenericNotification,
		GenericRequest,
		GenericResponse,
		RpcError,
		mountain_service_server::MountainService,
	},
};

/// The concrete implementation of the `MountainService` gRPC service.
pub struct MountainVinegRPCService {
	ApplicationHandle:AppHandle,
	RunTime:Arc<ApplicationRunTime>,
}

impl MountainVinegRPCService {
	/// Creates a new instance of the Mountain gRPC service.
	pub fn Create(ApplicationHandle:AppHandle, RunTime:Arc<ApplicationRunTime>) -> Self {
		info!("[MountainVinegRPCService] New instance created.");
		Self { ApplicationHandle, RunTime }
	}
}

#[tonic::async_trait]
impl MountainService for MountainVinegRPCService {
	/// Handles generic request-response RPCs from Cocoon.
	async fn process_cocoon_request(
		&self,
		request:Request<GenericRequest>,
	) -> Result<Response<GenericResponse>, Status> {
		let RequestData = request.into_inner();
		let MethodName = RequestData.method;
		let RequestIdentifier = RequestData.request_id;

		info!(
			"[VineServer] Received gRPC Request [ID: {}]: Method='{}'",
			RequestIdentifier, MethodName
		);

		let ParametersValue:Value = match serde_json::from_slice(&RequestData.params) {
			Ok(v) => v,
			Err(e) => {
				let msg = format!("Failed to deserialize parameters for method '{}': {}", MethodName, e);
				error!("{}", msg);
				return Ok(Response::new(GenericResponse {
					request_id:RequestIdentifier,
					result:vec![],
					error:Some(RpcError { message:msg, code:-32700, data:vec![] }),
				}));
			},
		};
		trace!("[VineServer] Params for [ID: {}]: {:?}", RequestIdentifier, ParametersValue);

		let DispatchResult = Track::DispatchLogic::DispatchSidecarRequest(
			self.ApplicationHandle.clone(),
			self.RunTime.clone(),
			"cocoon-main".to_string(), // In the future, this could come from connection metadata.
			MethodName.clone(),
			ParametersValue,
		)
		.await;

		match DispatchResult {
			Ok(SuccessfulResult) => {
				let ResultBytes = serde_json::to_vec(&SuccessfulResult).unwrap_or_else(|e| {
					error!("Failed to serialize successful result for '{}': {}", MethodName, e);
					b"null".to_vec()
				});
				Ok(Response::new(GenericResponse {
					request_id:RequestIdentifier,
					result:ResultBytes,
					error:None,
				}))
			},
			Err(ErrorString) => {
				Ok(Response::new(GenericResponse {
					request_id:RequestIdentifier,
					result:vec![],
					error:Some(RpcError {
						message:ErrorString,
						code:-32000, // JSON-RPC Generic Server Error
						data:vec![],
					}),
				}))
			},
		}
	}

	/// Handles generic fire-and-forget notifications from Cocoon.
	async fn send_cocoon_notification(&self, request:Request<GenericNotification>) -> Result<Response<Empty>, Status> {
		let NotificationData = request.into_inner();
		let MethodName = NotificationData.method;
		info!("[VineServer] Received gRPC Notification: Method='{}'", MethodName);

		// TODO: A full implementation would route these notifications to a
		// dedicated handler for processing status updates, etc. For now, we
		// just log and acknowledge.
		// For example:
		// let params: Value = serde_json::from_slice(...)?;
		// NotificationHandler::Handle(MethodName, params).await;

		Ok(Response::new(Empty {}))
	}

	/// Handles a request from Cocoon to cancel a long-running operation.
	async fn cancel_operation(&self, _request:Request<CancelOperationRequest>) -> Result<Response<Empty>, Status> {
		info!("[VineServer] Received CancelOperation request, but cancellation is not yet implemented.");
		// A full implementation would map the request_id_to_cancel to a
		// CancellationToken and trigger it.
		Ok(Response::new(Empty {}))
	}
}
