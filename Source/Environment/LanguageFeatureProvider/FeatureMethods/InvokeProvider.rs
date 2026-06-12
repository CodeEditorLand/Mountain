//! Invokes a registered language provider over reverse-RPC using the
//! `$provide{ProviderType}` method-name convention.
//!
//! The forward runs through [`ForwardCancellable`]: the Vine call executes on
//! a detached task guarded by a cancel-on-drop watch signal. When the calling
//! future is dropped mid-flight (typed-rail `CancelOperation` firing
//! `CocoonServiceImpl::RunCancellable`'s token, a gRPC peer disconnect, or
//! any caller-side `select!` loss), the guard flips the signal and the
//! detached task delivers `CocoonService.CancelOperation` for the wire
//! request id before exiting, so the side-car aborts the provider instead of
//! computing a result nobody is waiting for. A side-car timeout inside the
//! Vine client likewise fires `CancelOperation`.

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::watch;
use ::Vine::{Client::SendRequest::FnCancellable, Error::VineError};

use dashmap::DashMap;

use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

/// Per-forward timeout, matching the prior `SendRequestToSideCar` budget.
const FORWARD_TIMEOUT_MILLISECONDS:u64 = 5000;

pub(crate) async fn Fn(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let rpc_method = format!("$provide{}", registration.ProviderType.to_string());

	// Extract renderer-supplied requestId from the fourth argument if present.
	let RequestIdentifier = arguments
		.get(3)
		.and_then(|v| v.as_str())
		.filter(|s| !s.is_empty())
		.map(String::from);

	let Cancellations = environment
		.ApplicationState
		.Feature
		.LanguageProviderCancellations
		.clone();

	ForwardCancellable(
		registration.SideCarIdentifier.clone(),
		rpc_method,
		json!(arguments),
		Cancellations,
		RequestIdentifier,
	)
	.await
}

/// Flips the watch signal on drop unless disarmed, so a dropped forward
/// future still cancels the in-flight side-car request.
struct CancelOnDrop {
	Sender:watch::Sender<bool>,

	Armed:bool,
}

impl Drop for CancelOnDrop {
	fn drop(&mut self) {
		if self.Armed {
			let _ = self.Sender.send(true);
		}
	}
}

/// Sends `Method` to `SideCarIdentifier` on a detached task wired to a
/// cancel-on-drop guard; dropping the returned future delivers
/// `CocoonService.CancelOperation` for the allocated wire request id.
///
/// When `RequestIdentifier` is `Some`, the cancel sender is also stored in
/// `Cancellations` so that `language:cancelRequest` can flip it externally
/// before the future is dropped.
pub(crate) async fn ForwardCancellable(
	SideCarIdentifier:String,

	Method:String,

	Arguments:Value,

	Cancellations:Arc<DashMap<String, watch::Sender<bool>>>,

	RequestIdentifier:Option<String>,
) -> Result<Value, CommonError> {
	let (CancelSender, CancelReceiver) = watch::channel(false);

	// If the renderer provided a request identifier, register the sender
	// so that `language:cancelRequest` can trigger cancellation externally.
	if let Some(ref Id) = RequestIdentifier {
		Cancellations.insert(Id.clone(), CancelSender.clone());
	}

	let MethodForLog = Method.clone();
	let SideCarForLog = SideCarIdentifier.clone();

	let ForwardTask = tokio::spawn(async move {
		FnCancellable(
			&SideCarIdentifier,
			Method,
			Arguments,
			FORWARD_TIMEOUT_MILLISECONDS,
			CancelReceiver,
		)
		.await
	});

	let mut Guard = CancelOnDrop { Sender:CancelSender, Armed:true };

	let Outcome = ForwardTask.await;

	Guard.Armed = false;

	// Remove the cancellation entry now that the forward has finished.
	if let Some(ref Id) = RequestIdentifier {
		Cancellations.remove(Id);
	}

	match Outcome {
		Ok(Ok(Response)) => Ok(Response),

		Ok(Err(VineError::RequestCanceled { SideCarIdentifier, MethodName })) => {
			Err(CommonError::IPCError {
				Description:format!("request canceled: {}::{}", SideCarIdentifier, MethodName),
			})
		},

		Ok(Err(Error)) => Err(CommonError::IPCError { Description:Error.to_string() }),

		Err(JoinError) => {
			dev_log!(
				"grpc",
				"warn: [InvokeProvider] forward task for '{}::{}' failed to join: {}",
				SideCarForLog,
				MethodForLog,
				JoinError
			);

			Err(CommonError::IPCError { Description:format!("forward task join error: {}", JoinError) })
		},
	}
}
