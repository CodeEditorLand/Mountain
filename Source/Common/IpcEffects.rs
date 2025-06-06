// File: Common/IpcEffect.rs
// Defines the IpcProvider trait and associated effects for Inter-Process
// Communication. This provides a standardized way to send requests and
// notifications to sidecar processes.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	IpcDto::ProxyConfiguration,
	Runtime::AppRuntimeTrait,
}; // Assuming the DTO is in a dedicated module

/// A trait for environments that can facilitate inter-process communication.
#[async_trait]
pub trait IpcProvider: Environment {
	/// Sends a fire-and-forget notification to a sidecar process.
	async fn SendNotificationToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
	) -> Result<(), CommonError>;

	/// Sends a request to a sidecar process and awaits a response.
	async fn SendRequestToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
		TimeoutMilliseconds:u64,
	) -> Result<Value, CommonError>;
}

/// Creates an effect to send a notification to a sidecar.
pub fn SendNotification<RuntimeAccessType>(
	SidecarIdentifier:String,
	Method:String,
	Parameters:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn IpcProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let SidecarIdentifierClone = SidecarIdentifier.clone();
		let MethodClone = Method.clone();
		let ParametersClone = Parameters.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn IpcProvider> = Environment.require();
			Provider
				.SendNotificationToSidecar(SidecarIdentifierClone, MethodClone, ParametersClone)
				.await
		})
	}))
}

/// Creates an effect to send a request to a sidecar.
pub fn SendRequest<RuntimeAccessType>(
	SidecarIdentifier:String,
	Method:String,
	Parameters:Value,
	TimeoutMilliseconds:u64,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Value>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn IpcProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let SidecarIdentifierClone = SidecarIdentifier.clone();
		let MethodClone = Method.clone();
		let ParametersClone = Parameters.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn IpcProvider> = Environment.require();
			Provider
				.SendRequestToSidecar(SidecarIdentifierClone, MethodClone, ParametersClone, TimeoutMilliseconds)
				.await
		})
	}))
}

/// Creates an effect to establish a connection (e.g., handshake) with a
/// sidecar. This is implemented as a simple notification for signaling
/// readiness.
pub fn EstablishConnection<RuntimeAccessType>(
	SidecarIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn IpcProvider>>, {
	SendNotification(SidecarIdentifier, "internal_ping_handshake".to_string(), Value::Null)
}

/// Creates an effect to proxy a generic call to a sidecar.
/// This is a higher-level abstraction over `SendRequest`.
pub fn ProxyCallToSidecar<RuntimeAccessType>(
	TargetSidecarIdentifier:String,
	CallData:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Value>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn IpcProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let TargetSidecarIdentifierClone = TargetSidecarIdentifier.clone();
		let CallDataClone = CallData.clone();
		Box::pin(async move {
			let MethodString = CallDataClone
				.get("method")
				.and_then(Value::as_str)
				.ok_or_else(|| {
					CommonError::InvalidArg {
						ArgumentName:"CallData.method".to_string(),
						Reason:"Expected a 'method' string field in CallData for proxying.".to_string(),
					}
				})?
				.to_string();
			let ParametersValue = CallDataClone.get("params").cloned().unwrap_or(Value::Null);
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn IpcProvider> = Environment.require();
			let Timeout = 30000; // Default timeout for proxied calls
			Provider
				.SendRequestToSidecar(TargetSidecarIdentifierClone, MethodString, ParametersValue, Timeout)
				.await
		})
	}))
}
