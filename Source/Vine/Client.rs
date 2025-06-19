//! # Vine gRPC Client
//!
//! This module contains the logic for the gRPC client that connects to the
//! `Cocoon` sidecar.

use std::{collections::HashMap, sync::Arc, time::Duration};

use lazy_static::lazy_static;
use log::{debug, error, info};
use parking_lot::Mutex;
use serde_json::Value;
use tokio::time::timeout;
use tonic::transport::Channel;

use super::{
	Error::VineError,
	Generated::{GenericNotification, GenericRequest, cocoon_service_client::CocoonServiceClient},
};

type CocoonClient = CocoonServiceClient<Channel>;

lazy_static! {
	static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Establishes a gRPC client connection to a sidecar process.
pub async fn ConnectToSidecar(SidecarIdentifier:String, Address:String) -> Result<(), VineError> {
	debug!(
		"[VineClient] Attempting to connect to sidecar '{}' at '{}'",
		SidecarIdentifier, Address
	);
	let Channel = Channel::from_shared(format!("http://{}", Address))?.connect().await?;
	let Client = CocoonServiceClient::new(Channel);

	SIDECAR_CLIENTS.lock().insert(SidecarIdentifier.clone(), Client);
	info!("[VineClient] Successfully connected to sidecar '{}'", SidecarIdentifier);
	Ok(())
}

/// Retrieves a clone of a connected client from the shared map.
fn GetClient(SidecarIdentifier:&str) -> Result<CocoonClient, VineError> {
	SIDECAR_CLIENTS.lock().get(SidecarIdentifier).cloned().ok_or_else(|| {
		VineError::ClientChannelError {
			SidecarIdentifier:SidecarIdentifier.to_string(),
			Details:"Client not found in connection map.".to_string(),
		}
	})
}

/// Sends a fire-and-forget notification to a connected sidecar.
pub async fn SendNotification(SidecarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {
	let mut Client = GetClient(&SidecarIdentifier)?;
	let ParametersBytes = serde_json::to_vec(&Parameters)?;
	let Request = tonic::Request::new(GenericNotification { method:Method, params:ParametersBytes });
	Client.send_mountain_notification(Request).await?;
	Ok(())
}

/// Sends a request to a sidecar and awaits a response, with a timeout.
pub async fn SendRequest(
	SidecarIdentifier:&str,
	Method:String,
	Parameters:Value,
	TimeoutMilliseconds:u64,
) -> Result<Value, VineError> {
	let mut Client = GetClient(SidecarIdentifier)?;
	let ParametersBytes = serde_json::to_vec(&Parameters)?;
	let RequestIdentifier = rand::random::<u64>();

	let Request = tonic::Request::new(GenericRequest {
		request_id:RequestIdentifier,
		method:Method.clone(),
		params:ParametersBytes,
	});

	let RequestFuture = Client.process_mountain_request(Request);
	match timeout(Duration::from_millis(TimeoutMilliseconds), RequestFuture).await {
		Ok(Ok(Response)) => {
			let ResponseData = Response.into_inner();
			if let Some(Error) = ResponseData.error {
				Err(VineError::gRPCRequestFailed {
					SidecarIdentifier:SidecarIdentifier.to_string(),
					MethodName:Method,
					StatusCode:Error.code.to_string(),
					StatusMessage:Error.message,
				})
			} else {
				Ok(serde_json::from_slice(&ResponseData.result)?)
			}
		},
		Ok(Err(Status)) => {
			Err(VineError::gRPCRequestFailed {
				SidecarIdentifier:SidecarIdentifier.to_string(),
				MethodName:Method,
				StatusCode:Status.code().to_string(),
				StatusMessage:Status.message().to_string(),
			})
		},
		Err(_) => {
			Err(VineError::RequestTimeout {
				SidecarIdentifier:SidecarIdentifier.to_string(),
				MethodName:Method,
				TimeoutMilliseconds,
			})
		},
	}
}
