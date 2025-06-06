
// Defines the Cocoon-specific implementation of the RPCProtocol, which is
// responsible for dispatching incoming RPC calls from Mountain to the correct
// local service shims.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use log::{debug, error, trace, warn};
use vs_base_common_buffer::VSBuffer;
use vs_base_common_uriipc::IURITransformer;
use vs_base_parts_ipc_common_ipc::IMessagePassingProtocol;
use vs_workbench_api_common_exthost_protocol::ExtHostContext;
use vs_workbench_services_extensions_common_rpcprotocol::{IRPCProtocolLogger, RPCProtocol};

use crate::Ipc::Rpc::ServiceIdentifierMap::SERVICE_ID_TO_EXT_HOST_CONTEXT_MAP; // PascalCased map

const MESSAGE_TYPE_REQUEST:u8 = 1;
const MESSAGE_TYPE_NOTIFICATION:u8 = 3; // From the original file, VS Code uses 3 for notifications in some contexts.

/// A wrapper around VS Code's `RPCProtocol` to add Cocoon-specific dispatch
/// logic.
pub struct CocoonRpcProtocol {
	// The underlying RPCProtocol instance that handles message marshalling and proxying.
	Rpc:Arc<RPCProtocol>,
}

impl CocoonRpcProtocol {
	/// Creates a new `CocoonRpcProtocol`.
	pub fn New(
		Protocol:Arc<dyn IMessagePassingProtocol + Send + Sync>,
		Logger:Option<Arc<dyn IRPCProtocolLogger + Send + Sync>>,
		Transformer:Option<Arc<dyn IURITransformer + Send + Sync>>,
	) -> Self {
		let RpcInstance = Arc::new(RPCProtocol::new(Protocol, Logger, Transformer));
		debug!("[CocoonRpcProtocol] New instance created.");
		Self { Rpc:RpcInstance }
	}

	/// Dispatches a call received from Mountain to the appropriate local
	/// service shim.
	pub async fn DispatchCallToLocalTarget(
		&self,
		FullMethodName:String,
		ArgumentArray:Vec<serde_json::Value>,
		MountainRequestIdentifier:Option<u64>,
	) -> Result<serde_json::Value, String> {
		let OperationTypeLog = if MountainRequestIdentifier.is_some() { "Request" } else { "Notification" };
		debug!(
			"[CocoonRpcProtocol] DispatchCallToLocalTarget ({}): FullMethod='{}', ArgumentCount={}, MountainReqID={:?}",
			OperationTypeLog,
			FullMethodName,
			ArgumentArray.len(),
			MountainRequestIdentifier
		);

		let Parts:Vec<&str> = FullMethodName.splitn(2, '.').collect();
		if Parts.len() != 2 || Parts[0].is_empty() || Parts[1].is_empty() || !Parts[1].starts_with('$') {
			let ErrorMessage = format!(
				"[CocoonRpcProtocol] Invalid local dispatch format: Method name '{}' must be \
				 'ServiceName.$methodName'.",
				FullMethodName
			);
			error!("{}", ErrorMessage);
			return Err(ErrorMessage);
		}

		let ServiceIdentifierString = Parts[0].to_string();
		let MethodName = Parts[1].to_string();

		let ContextVariant = SERVICE_ID_TO_EXT_HOST_CONTEXT_MAP
			.get(&ServiceIdentifierString)
			.cloned()
			.ok_or_else(|| {
				let ErrorMessage = format!(
					"[CocoonRpcProtocol] No ExtHostContext variant mapped for service string ID: '{}'.",
					ServiceIdentifierString
				);
				error!("{}", ErrorMessage);
				ErrorMessage
			})?;

		let LocalInstance = self.Rpc.get_local_service_instance(ContextVariant).ok_or_else(|| {
			let ErrorMessage = format!(
				"[CocoonRpcProtocol] No local instance found for service ExtHostContext::{:?} (from '{}').",
				ContextVariant, ServiceIdentifierString
			);
			error!("{}", ErrorMessage);
			ErrorMessage
		})?;

		// This part is conceptual in Rust. The actual dynamic method call would require
		// a more complex dispatch mechanism, likely involving a `match` on
		// `ServiceIdentifierString` that downcasts the `LocalInstance` and calls the
		// appropriate method.
		debug!(
			"[CocoonRpcProtocol] Conceptually invoking local shim method: {}.{}",
			ServiceIdentifierString, MethodName
		);

		// Placeholder for dynamic dispatch logic.
		if MountainRequestIdentifier.is_some() {
			Err(format!(
				"Dynamic request dispatch for {}.{} is a stub in Rust.",
				ServiceIdentifierString, MethodName
			))
		} else {
			Ok(serde_json::Value::Null)
		}
	}

	/// Processes a raw message buffer received from the underlying message
	/// passing protocol.
	pub fn ProcessReceivedMountainRpcBuffer(&self, MessageBuffer:VSBuffer) {
		if self.Rpc.is_disposed() {
			warn!("[CocoonRpcProtocol] processReceivedMountainRpcBuffer called on disposed protocol. Ignoring.");
			return;
		}
		trace!(
			"[CocoonRpcProtocol] Processing received Mountain RPC buffer (len: {}) via parent _receiveMessage.",
			MessageBuffer.byte_length()
		);
		self.Rpc.receive_ipc_message(MessageBuffer);
	}

	// Exposing underlying RPCProtocol methods
	pub fn GetProxy<T:Send + Sync + 'static>(&self, Identifier:ExtHostContext) -> Arc<T> {
		self.Rpc.get_proxy(Identifier)
	}

	pub fn Set<T:Send + Sync + 'static>(&self, Identifier:ExtHostContext, Instance:Arc<T>) {
		self.Rpc.set_local_service_instance(Identifier, Instance);
	}

	pub fn Dispose(&self) {
		self.Rpc.dispose();
		debug!("[CocoonRpcProtocol] Disposed.");
	}
}
