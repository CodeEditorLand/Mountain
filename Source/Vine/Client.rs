//! # Vine Client
//!
//! Provides a simplified, thread-safe client for communicating with a `Cocoon`
//! sidecar process via gRPC. It manages a shared pool of connections.

use std::{
	collections::{HashMap, hash_map::DefaultHasher},
	hash::{Hash, Hasher},
	sync::Arc,
	time::Duration,
};

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
	let endpoint = format!("http://{}", Address);
	let channel = Channel::from_shared(endpoint)?.connect().await?;
	let client = CocoonServiceClient::new(channel);
	SIDECAR_CLIENTS.lock().insert(SidecarIdentifier.clone(), client);
	info!("[VineClient] Successfully connected to sidecar '{}'.", SidecarIdentifier);
	Ok(())
}

/// Sends a fire-and-forget notification to a sidecar.
pub async fn SendNotification(SidecarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {
	let mut client = {
		let guard = SIDECAR_CLIENTS.lock();
		guard.get(&SidecarIdentifier).cloned()
	};

	if let Some(ref mut client) = client {
		let request = GenericNotification { method:Method, params:to_vec(&Parameters)? };
		client.send_mountain_notification(request).await?;
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
	let mut client = {
		let guard = SIDECAR_CLIENTS.lock();
		guard.get(SidecarIdentifier).cloned()
	};

	if let Some(ref mut client) = client {
		let mut hasher = DefaultHasher::new();
		uuid::Uuid::new_v4().hash(&mut hasher);
		let request_id = hasher.finish();

		let request = GenericRequest { request_id, method:Method.clone(), params:to_vec(&Parameters)? };

		let future = client.process_mountain_request(request);

		match timeout(Duration::from_millis(TimeoutMilliseconds), future).await {
			Ok(Ok(response)) => {
				let response_data = response.into_inner();
				if let Some(rpc_error) = response_data.error {
					error!(
						"[VineClient] Received RPC error from sidecar '{}': {}",
						SidecarIdentifier, rpc_error.message
					);
					Err(VineError::RpcError(rpc_error.message))
				} else {
					let deserialized_value = from_slice(&response_data.result)?;
					Ok(deserialized_value)
				}
			},
			Ok(Err(status)) => {
				error!(
					"[VineClient] gRPC status error from sidecar '{}': {}",
					SidecarIdentifier, status
				);
				Err(VineError::from(status))
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
