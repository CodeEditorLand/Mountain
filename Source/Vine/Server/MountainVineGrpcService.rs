
// Defines the gRPC service implementation for Mountain. This service listens
// for incoming requests and notifications from the Cocoon sidecar and
// dispatches them to the appropriate handlers within the Mountain application.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use serde_json::{Value as JsonValue, json};
use tauri::{AppHandle, Manager, State, Window, Wry};
use tonic::{Request, Response, Status};

use crate::Track; // Assuming Track contains the main dispatch logic
use crate::Vine::VineGrpcPb::{
	Empty as GrpcEmpty, // Using an alias for clarity
	GenericNotification,
	GenericRequest,
	GenericResponse,
	JsonValueWrapper,
	RpcError as GrpcErrorPayload,
	mountain_service_server::{MountainService, MountainServiceServer}, // Correct trait and struct names
};
use crate::{Handlers::ErrorUtils, Runtime::AppRuntime};

#[derive(Clone)]
pub struct MountainVineGrpcService {
	ApplicationHandle:AppHandle<Wry>,
}

impl MountainVineGrpcService {
	pub fn New(ApplicationHandle:AppHandle<Wry>) -> Self {
		info!("[MountainVineGrpcService] New instance created.");
		Self { ApplicationHandle }
	}
}

#[tonic::async_trait]
impl MountainService for MountainVineGrpcService {
	/// Processes a request from the Cocoon sidecar.
	async fn ProcessCocoonRequest(&self, Request:Request<GenericRequest>) -> Result<Response<GenericResponse>, Status> {
		let RequestData = request.into_inner();
		let CocoonRequestId = RequestData.request_id;
		let MethodName = RequestData.method.clone();
		let ParametersJsonValue = RequestData.params.map_or(JsonValue::Null, |Wrapper| Wrapper.value);

		info!(
			"[MountainVineGrpcService] ProcessCocoonRequest: ID={}, Method='{}'",
			CocoonRequestId, MethodName
		);
		trace!("[MountainVineGrpcService] Params: {:?}", ParametersJsonValue);

		let ApplicationRuntimeState:State<'_, Arc<AppRuntime>> = self.ApplicationHandle.state();
		if ApplicationRuntimeState.inner().is_none() {
			let ErrorMessage = "AppRuntime not available in Mountain.";
			error!("[MountainVineGrpcService] {}", ErrorMessage);
			return Ok(Response::new(GenericResponse {
				request_id:CocoonRequestId,
				result:None,
				error:Some(GrpcErrorPayload { code:-32000, message:ErrorMessage.to_string(), data:None }),
			}));
		}

		let MainWindow = self
			.ApplicationHandle
			.get_webview_window("main")
			.ok_or_else(|| Status::internal("Main application window not found for gRPC request dispatch."))?;

		// Construct the payload expected by the Track dispatcher
		let DispatchRequestPayload = json!({
			"method": MethodName,
			"params": ParametersJsonValue,
		});

		let SidecarIdentifier = "cocoon-main".to_string();

		match Track::DispatchSidecarRequest(
			self.ApplicationHandle.clone(),
			MainWindow,
			ApplicationRuntimeState,
			SidecarIdentifier,
			DispatchRequestPayload,
		)
		.await
		{
			Ok(SuccessfulResultValue) => {
				debug!(
					"[MountainVineGrpcService] Request ID={} Method='{}' succeeded.",
					CocoonRequestId, MethodName
				);
				Ok(Response::new(GenericResponse {
					request_id:CocoonRequestId,
					result:Some(JsonValueWrapper { value:SuccessfulResultValue }),
					error:None,
				}))
			},
			Err(JsonRpcErrorString) => {
				warn!(
					"[MountainVineGrpcService] Request ID={} Method='{}' failed: {}",
					CocoonRequestId, MethodName, JsonRpcErrorString
				);
				let RpcErrorObject:GrpcErrorPayload =
					match serde_json::from_str::<serde_json::Map<String, JsonValue>>(&JsonRpcErrorString) {
						Ok(Map) => {
							GrpcErrorPayload {
								code:Map.get("code").and_then(JsonValue::as_i64).map_or(-32000, |c| c as i32),
								message:Map
									.get("message")
									.and_then(JsonValue::as_str)
									.unwrap_or(&JsonRpcErrorString)
									.to_string(),
								data:Map.get("data").map(|d| JsonValueWrapper { value:d.clone() }),
							}
						},
						Err(_) => GrpcErrorPayload { code:-32001, message:JsonRpcErrorString, data:None },
					};
				Ok(Response::new(GenericResponse {
					request_id:CocoonRequestId,
					result:None,
					error:Some(RpcErrorObject),
				}))
			},
		}
	}

	/// Processes a fire-and-forget notification from the Cocoon sidecar.
	async fn SendCocoonNotification(
		&self,
		Request:Request<GenericNotification>,
	) -> Result<Response<GrpcEmpty>, Status> {
		let NotificationData = request.into_inner();
		let MethodName = NotificationData.method.clone();
		let ParametersJsonValue = NotificationData.params.map_or(JsonValue::Null, |Wrapper| Wrapper.value);

		info!("[MountainVineGrpcService] SendCocoonNotification: Method='{}'", MethodName);
		trace!("[MountainVineGrpcService] Notification Params: {:?}", ParametersJsonValue);

		// Dispatching a notification is similar to a request but we don't expect a
		// result. We still need the runtime and window context for the dispatcher.
		let ApplicationRuntimeState:State<'_, Arc<AppRuntime>> = self.ApplicationHandle.state();
		let MainWindow = self
			.ApplicationHandle
			.get_webview_window("main")
			.ok_or_else(|| Status::internal("Main application window not found for gRPC notification."))?;

		let DispatchNotificationPayload = json!({
			"method": MethodName,
			"params": ParametersJsonValue,
		});

		let SidecarIdentifier = "cocoon-main".to_string();

		// Spawn a task to handle the notification so we can return immediately.
		let AppHandleClone = self.ApplicationHandle.clone();
		tokio::spawn(async move {
			match Track::DispatchSidecarRequest(
				AppHandleClone,
				MainWindow,
				ApplicationRuntimeState,
				SidecarIdentifier,
				DispatchNotificationPayload,
			)
			.await
			{
				Ok(_) => {
					debug!(
						"[MountainVineGrpcService] Notification '{}' processed successfully.",
						MethodName
					)
				},
				Err(Error) => {
					error!(
						"[MountainVineGrpcService] Error processing notification '{}': {}",
						MethodName, Error
					)
				},
			}
		});

		Ok(Response::new(GrpcEmpty {}))
	}
}
