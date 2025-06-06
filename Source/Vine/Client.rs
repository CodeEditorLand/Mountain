// File: Vine/Client.rs
// Defines the gRPC client for Mountain to communicate with the Cocoon sidecar.
// It handles connection management, request/notification sending, and
// cancellation.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, sync::Arc, time::Duration};

use log::{debug, error, info, trace, warn};
use serde_json::Value as JsonValue;
use tokio::sync::Mutex as TokioMutex;
use tonic::transport::{Channel, Endpoint};

use crate::Vine::{
	VineError,
	VineGrpcPb::{
		CancelOperationRequest,
		GenericNotification,
		GenericRequest,
		JsonValueWrapper,
		RpcDataPayload,
		cocoon_service_client::CocoonServiceClient,
	},
};

type ActiveCocoonClientMap = HashMap<String, CocoonServiceClient<Channel>>;
static ACTIVE_COCOON_CLIENTS:Lazy<Arc<TokioMutex<ActiveCocoonClientMap>>> =
	Lazy::new(|| Arc::new(TokioMutex::new(HashMap::new())));

/// Establishes a gRPC connection to a Cocoon sidecar service and stores the
/// client.
pub async fn ConnectToCocoonService(SidecarIdentifier:String, CocoonAddress:String) -> Result<(), VineError> {
	info!(
		"[VineClient] Connecting to Cocoon '{}' at: {}",
		SidecarIdentifier, CocoonAddress
	);

	let EndpointAddress = {
		#[cfg(unix)]
		{
			Endpoint::from_shared(format!("unix:{}", CocoonAddress))
		}
		#[cfg(windows)]
		{
			Endpoint::from_shared(format!("windows-named-pipe:{}", CocoonAddress))
		}
		#[cfg(not(any(unix, windows)))]
		{
			return Err(VineError::ClientChannelError {
				SidecarIdentifier,
				Details:"Unsupported gRPC transport platform.".to_string(),
			});
		}
	}
	.map_err(|Error| {
		VineError::ClientChannelError {
			SidecarIdentifier:SidecarIdentifier.clone(),
			Details:format!("Invalid endpoint URI: {}", Error),
		}
	})?;

	let ConnectTimeout = Duration::from_secs(10);
	let ChannelInstance = match tokio::time::timeout(ConnectTimeout, EndpointAddress.connect()).await {
		Ok(Ok(Channel)) => Channel,
		Ok(Err(Error)) => {
			return Err(VineError::ClientChannelError {
				SidecarIdentifier,
				Details:format!("Failed to connect to gRPC endpoint: {}", Error),
			});
		},
		Err(_) => {
			return Err(VineError::ClientChannelError {
				SidecarIdentifier,
				Details:format!("Timeout connecting to gRPC endpoint after {}s", ConnectTimeout.as_secs()),
			});
		},
	};

	let ClientInstance = CocoonServiceClient::new(ChannelInstance);
	let mut ClientsMapGuard = ACTIVE_COCOON_CLIENTS.lock().await;
	ClientsMapGuard.insert(SidecarIdentifier.clone(), ClientInstance);
	info!(
		"[VineClient] Connected and registered gRPC client for Cocoon: '{}'",
		SidecarIdentifier
	);
	Ok(())
}

/// Removes a gRPC client from the active pool.
pub async fn UnregisterCocoonClient(SidecarIdentifier:&str) {
	let mut ClientsMapGuard = ACTIVE_COCOON_CLIENTS.lock().await;
	if ClientsMapGuard.remove(SidecarIdentifier).is_some() {
		info!("[VineClient] Unregistered gRPC client for Cocoon: '{}'", SidecarIdentifier);
	} else {
		debug!(
			"[VineClient] No active gRPC client found to unregister for Cocoon: '{}'",
			SidecarIdentifier
		);
	}
}

/// Retrieves a cloned gRPC client instance for a given sidecar identifier.
async fn GetClonedClient(SidecarIdentifier:&str) -> Result<CocoonServiceClient<Channel>, VineError> {
	let ClientsMapGuard = ACTIVE_COCOON_CLIENTS.lock().await;
	ClientsMapGuard.get(SidecarIdentifier).cloned().ok_or_else(|| {
		VineError::ClientChannelError {
			SidecarIdentifier:SidecarIdentifier.to_string(),
			Details:"Client for sidecar not found or not connected.".to_string(),
		}
	})
}

/// Sends a request to a sidecar and awaits a response.
pub async fn SendRequest(
	TargetSidecarIdentifier:&str,
	RequestIdentifier:u64,
	MethodName:String,
	ParametersValue:JsonValue,
	TimeoutMilliseconds:u64,
) -> Result<JsonValue, VineError> {
	trace!(
		"[Vine SendRequest] To '{}': ID={}, Method='{}'",
		TargetSidecarIdentifier, RequestIdentifier, MethodName
	);
	let mut Client = GetClonedClient(TargetSidecarIdentifier).await?;
	let GrpcRequest = GenericRequest {
		request_id:RequestIdentifier,
		method:MethodName.clone(),
		params:Some(JsonValueWrapper { value:ParametersValue }),
	};
	let RequestFuture = Client.process_mountain_request(tonic::Request::new(GrpcRequest));

	match tokio::time::timeout(Duration::from_millis(TimeoutMilliseconds), RequestFuture).await {
		Ok(Ok(TonicResponse)) => {
			let GrpcResponse = TonicResponse.into_inner();
			if let Some(RpcErrorPayload) = GrpcResponse.error {
				warn!(
					"[Vine SendRequest] Sidecar '{}' returned error for '{}' (ID {}): Code={}, Msg='{}'",
					TargetSidecarIdentifier,
					MethodName,
					RequestIdentifier,
					RpcErrorPayload.code,
					RpcErrorPayload.message
				);
				Err(VineError::GrpcRequestFailed {
					SidecarIdentifier:TargetSidecarIdentifier.to_string(),
					MethodName,
					StatusCode:RpcErrorPayload.code.to_string(),
					StatusMessage:RpcErrorPayload.message,
				})
			} else {
				debug!(
					"[Vine SendRequest] Success for '{}' (ID {}) from '{}'.",
					MethodName, RequestIdentifier, TargetSidecarIdentifier
				);
				Ok(GrpcResponse.result.map_or(JsonValue::Null, |Wrapper| Wrapper.value))
			}
		},
		Ok(Err(Status)) => {
			error!(
				"[Vine SendRequest] gRPC transport error for '{}' to '{}': {}",
				MethodName, TargetSidecarIdentifier, Status
			);
			Err(VineError::GrpcRequestFailed {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName,
				StatusCode:Status.code().to_string(),
				StatusMessage:Status.message().to_string(),
			})
		},
		Err(_) => {
			error!(
				"[Vine SendRequest] Timeout after {}ms for '{}' to '{}'.",
				TimeoutMilliseconds, MethodName, TargetSidecarIdentifier
			);
			Err(VineError::RequestTimeout {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName,
				TimeoutDurationMilliseconds:TimeoutMilliseconds,
			})
		},
	}
}

/// Sends a fire-and-forget notification to a sidecar.
pub async fn SendNotification(
	TargetSidecarIdentifier:&str,
	MethodName:String,
	ParametersValue:JsonValue,
) -> Result<(), VineError> {
	trace!(
		"[Vine SendNotification] To '{}': Method='{}'",
		TargetSidecarIdentifier, MethodName
	);
	let mut Client = GetClonedClient(TargetSidecarIdentifier).await?;
	let GrpcNotification = GenericNotification {
		method:MethodName.clone(),
		params:Some(JsonValueWrapper { value:ParametersValue }),
	};
	let NotificationTimeout = Duration::from_secs(5);
	match tokio::time::timeout(
		NotificationTimeout,
		Client.send_mountain_notification(tonic::Request::new(GrpcNotification)),
	)
	.await
	{
		Ok(Ok(_)) => Ok(()),
		Ok(Err(Status)) => {
			error!(
				"[Vine SendNotification] gRPC transport error for '{}' to '{}': {}",
				MethodName, TargetSidecarIdentifier, Status
			);
			Err(VineError::GrpcRequestFailed {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName,
				StatusCode:Status.code().to_string(),
				StatusMessage:Status.message().to_string(),
			})
		},
		Err(_) => {
			error!(
				"[Vine SendNotification] Timeout for '{}' to '{}'.",
				MethodName, TargetSidecarIdentifier
			);
			Err(VineError::RequestTimeout {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName,
				TimeoutDurationMilliseconds:NotificationTimeout.as_millis() as u64,
			})
		},
	}
}

/// Sends raw binary data to a sidecar.
pub async fn SendRpcData(TargetSidecarIdentifier:&str, BufferData:Vec<u8>) -> Result<(), VineError> {
	trace!(
		"[Vine SendRpcData] To '{}', BufferLen={}",
		TargetSidecarIdentifier,
		BufferData.len()
	);
	let mut Client = GetClonedClient(TargetSidecarIdentifier).await?;
	let GrpcRpcData = RpcDataPayload { buffer:BufferData };
	let RpcDataTimeout = Duration::from_secs(10);
	match tokio::time::timeout(RpcDataTimeout, Client.send_rpc_data_to_cocoon(tonic::Request::new(GrpcRpcData))).await {
		Ok(Ok(_)) => Ok(()),
		Ok(Err(Status)) => {
			error!(
				"[Vine SendRpcData] gRPC error for RPCData to '{}': {}",
				TargetSidecarIdentifier, Status
			);
			Err(VineError::GrpcRequestFailed {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName:"SendRpcDataToCocoon".to_string(),
				StatusCode:Status.code().to_string(),
				StatusMessage:Status.message().to_string(),
			})
		},
		Err(_) => {
			error!("[Vine SendRpcData] Timeout for RPCData to '{}'.", TargetSidecarIdentifier);
			Err(VineError::RequestTimeout {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName:"SendRpcDataToCocoon".to_string(),
				TimeoutDurationMilliseconds:RpcDataTimeout.as_millis() as u64,
			})
		},
	}
}

/// Sends a cancellation request for a previously sent request.
pub async fn SendCancel(TargetSidecarIdentifier:&str, RequestIdentifierToCancel:u64) -> Result<(), VineError> {
	info!(
		"[Vine SendCancel] To '{}', CancellingOpID={}",
		TargetSidecarIdentifier, RequestIdentifierToCancel
	);
	let mut Client = GetClonedClient(TargetSidecarIdentifier).await?;
	let GrpcCancel = CancelOperationRequest { request_id_to_cancel:RequestIdentifierToCancel };
	let CancelTimeout = Duration::from_secs(5);
	match tokio::time::timeout(CancelTimeout, Client.cancel_cocoon_operation(tonic::Request::new(GrpcCancel))).await {
		Ok(Ok(_)) => Ok(()),
		Ok(Err(Status)) => {
			error!(
				"[Vine SendCancel] gRPC error for Cancel (OpID={}) to '{}': {}",
				RequestIdentifierToCancel, TargetSidecarIdentifier, Status
			);
			Err(VineError::GrpcRequestFailed {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName:"CancelCocoonOperation".to_string(),
				StatusCode:Status.code().to_string(),
				StatusMessage:Status.message().to_string(),
			})
		},
		Err(_) => {
			error!(
				"[Vine SendCancel] Timeout for Cancel (OpID={}) to '{}'.",
				RequestIdentifierToCancel, TargetSidecarIdentifier
			);
			Err(VineError::RequestTimeout {
				SidecarIdentifier:TargetSidecarIdentifier.to_string(),
				MethodName:"CancelCocoonOperation".to_string(),
				TimeoutDurationMilliseconds:CancelTimeout.as_millis() as u64,
			})
		},
	}
}
