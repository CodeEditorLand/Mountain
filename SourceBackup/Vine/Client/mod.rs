// @module client (Vine)
// @description This module contains the logic for the gRPC client that connects
// to the Cocoon sidecar.

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::Duration,
};

use lazy_static::lazy_static;
use log::{debug, error, warn};
use serde_json::Value;
use tokio::time::timeout;
use tonic::transport::Channel;

use crate::Vine::{
	error::VineError,
	generated::{cocoon_service_client::CocoonServiceClient, GenericNotification, GenericRequest},
};

type CocoonClient = CocoonServiceClient<Channel>;

lazy_static! {
	static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Establishes a gRPC client connection to a sidecar process.
/// This should be called after the sidecar process confirms it is ready.
pub async fn ConnectToSidecar(sidecar_id: String, address: String) -> Result<(), VineError> {
	debug!("[VineClient] Attempting to connect to sidecar '{}' at '{}'", sidecar_id, address);
	let channel = Channel::from_shared(format!("http://{}", address))?.connect().await?;
	let client = CocoonServiceClient::new(channel);

	let mut clients_guard = SIDECAR_CLIENTS.lock().map_err(VineError::from)?;
	clients_guard.insert(sidecar_id.clone(), client);

	info!("[VineClient] Successfully connected to sidecar '{}'", sidecar_id);
	Ok(())
}

/// Retrieves a clone of a connected client.
fn get_client(sidecar_id: &str) -> Result<CocoonClient, VineError> {
	let clients_guard = SIDECAR_CLIENTS.lock().map_err(VineError::from)?;
	clients_guard.get(sidecar_id).cloned().ok_or_else(|| VineError::ClientChannelError {
		sidecar_identifier: sidecar_id.to_string(),
		details: "Client not found in connection map.".to_string(),
	})
}

/// Sends a fire-and-forget notification to a connected sidecar.
pub async fn SendNotification(sidecar_id: String, method: String, params: Value) -> Result<(), VineError> {
	let mut client = get_client(&sidecar_id)?;
	let params_bytes = serde_json::to_vec(¶ms)?;

	let request = tonic::Request::new(GenericNotification { method, params: params_bytes });

	if let Err(e) = client.send_mountain_notification(request).await {
		error!("[VineClient] Failed to send notification to '{}': {}", sidecar_id, e);
		return Err(VineError::gRPCRequestFailed {
			sidecar_identifier: sidecar_id,
			method_name: "send_mountain_notification".into(),
			status_code: e.code().to_string(),
			status_message: e.message().to_string(),
		});
	}

	Ok(())
}

/// Sends a request to a sidecar and awaits a response, with a timeout.
pub async fn SendRequest(
	sidecar_id: String,
	method: String,
	params: Value,
	timeout_ms: u64,
) -> Result<Value, VineError> {
	let mut client = get_client(&sidecar_id)?;
	let params_bytes = serde_json::to_vec(¶ms)?;
	let request_id = rand::random::<u64>(); // Simple random ID

	let request = tonic::Request::new(GenericRequest { request_id, method: method.clone(), params: params_bytes });

	let request_future = client.process_mountain_request(request);
	match timeout(Duration::from_millis(timeout_ms), request_future).await {
		Ok(Ok(response)) => {
			let response_data = response.into_inner();
			if let Some(err) = response_data.error {
				Err(VineError::gRPCRequestFailed {
					sidecar_identifier: sidecar_id,
					method_name: method,
					status_code: err.code.to_string(),
					status_message: err.message,
				})
			} else {
				Ok(serde_json::from_slice(&response_data.result)?)
			}
		},
		Ok(Err(status)) => Err(VineError::gRPCRequestFailed {
			sidecar_identifier: sidecar_id,
			method_name: method,
			status_code: status.code().to_string(),
			status_message: status.message().to_string(),
		}),
		Err(_) => Err(VineError::RequestTimeout {
			sidecar_identifier: sidecar_id,
			method_name: method,
			timeout_milliseconds: timeout_ms,
		}),
	}
}
