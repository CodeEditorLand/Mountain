//! # Vine Client
//!
//! Provides a simplified, thread-safe client for communicating with a `Cocoon`
//! sidecar process via gRPC. It manages a shared pool of connections.

use std::{collections::HashMap, sync::Arc, time::Duration};

use lazy_static::lazy_static;
use log::{debug, error, info};
use parking_lot::Mutex;
use serde_json::{Value, from_slice, to_vec};
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

/// Establishes a gRPC connection to a sidecar process.
pub async fn ConnectToSidecar(SidecarIdentifier:String, Address:String) -> Result<(), VineError> {
	info!("[VineClient] Connecting to sidecar '{}' at '{}'...", SidecarIdentifier, Address);
	let Channel = Channel::from_shared(format!("http://{}", Address))?.connect().await?;
	let Client = CocoonServiceClient::new(Channel);
	SIDECAR_CLIENTS.lock().insert(SidecarIdentifier.clone(), Client);
	info!("[VineClient] Successfully connected to sidecar '{}'.", SidecarIdentifier);
	Ok(())
}

/// Sends a fire-and-forget notification to a sidecar.
pub async fn SendNotification(SidecarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {
	let mut Guard = SIDECAR_CLIENTS.lock();
	if let Some(Client) = Guard.get_mut(&SidecarIdentifier) {
		let Request = GenericNotification { method:Method, params:to_vec(&Parameters)? };
		Client.send_cocoon_notification(Request).await?;
		Ok(())
	} else {
		Err(VineError::ClientNotConnected(SidecarIdentifier))
	}
}

/// Sends a request to a sidecar and awaits a response.
pub async fn SendRequest(
	SidecarIdentifier:&str,
	Method:String,
	Parameters:Value,
	TimeoutMilliseconds:u64,
) -> Result<Value, VineError> {
	debug!(
		"[VineClient] Sending request '{}' to sidecar '{}'...",
		Method, SidecarIdentifier
	);
	let mut Guard = SIDECAR_CLIENTS.lock();
	if let Some(Client) = Guard.get_mut(SidecarIdentifier) {
		// Use a unique request ID for tracking.
		let RequestIdentifier = uuid::Uuid::new_v4().to_string();
		let Request = GenericRequest {
			request_id:RequestIdentifier.clone(),
			method:Method.clone(),
			params:to_vec(&Parameters)?,
		};

		let Future = Client.process_mountain_request(Request);

		match timeout(Duration::from_millis(TimeoutMilliseconds), Future).await {
			Ok(Ok(Response)) => {
				let ResponseData = Response.into_inner();
				if let Some(RPCError) = ResponseData.error {
					error!(
						"[VineClient] Received RPC error from sidecar '{}': {}",
						SidecarIdentifier, RPCError.message
					);
					Err(VineError::RPCError(RPCError.message))
				} else {
					let DeserializedValue = from_slice(&ResponseData.result)?;
					Ok(DeserializedValue)
				}
			},
			Ok(Err(Status)) => {
				error!(
					"[VineClient] gRPC status error from sidecar '{}': {}",
					SidecarIdentifier, Status
				);
				Err(VineError::from(Status))
			},
			Err(_) => {
				error!("[VineClient] Request to sidecar '{}' timed out.", SidecarIdentifier);
				Err(VineError::RequestTimeout {
					SidecarIdentifier:SidecarIdentifier.to_string(),
					MethodName:Method,
					TimeoutMilliseconds,
				})
			},
		}
	} else {
		Err(VineError::ClientNotConnected(SidecarIdentifier.to_string()))
	}
}
