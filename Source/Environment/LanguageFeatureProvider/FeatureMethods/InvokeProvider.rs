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
use tokio::sync::watch;
use ::Vine::{Client::SendRequest::FnCancellable, Error::VineError};

use crate::{ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO, dev_log};

/// Per-forward timeout, matching the prior `SendRequestToSideCar` budget.
const FORWARD_TIMEOUT_MILLISECONDS:u64 = 5000;

pub(crate) async fn Fn(
	_environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let rpc_method = format!("$provide{}", registration.ProviderType.to_string());

	ForwardCancellable(registration.SideCarIdentifier.clone(), rpc_method, json!(arguments)).await
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
pub(super) async fn ForwardCancellable(
	SideCarIdentifier:String,

	Method:String,

	Arguments:Value,
) -> Result<Value, CommonError> {
	let (CancelSender, CancelReceiver) = watch::channel(false);

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
