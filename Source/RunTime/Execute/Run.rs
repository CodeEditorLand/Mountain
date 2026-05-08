#![allow(non_snake_case)]

//! `ApplicationRunTimeTrait::Run` - submit an `ActionEffect` to the Echo
//! work-stealing scheduler and block on the oneshot reply.

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};
use Echo::Task::Priority::Priority;
use async_trait::async_trait;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

#[async_trait]
impl ApplicationRunTimeTrait for ApplicationRunTime {
	async fn Run<TCapabilityProvider, TError, TOutput>(
		&self,

		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
			Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		let (ResultSender, ResultReceiver) = tokio::sync::oneshot::channel::<Result<TOutput, TError>>();

		let CapabilityProvider:Arc<TCapabilityProvider> = self.Environment.Require();

		let Task = async move {
			let Result = Effect.Apply(CapabilityProvider).await;

			if ResultSender.send(Result).is_err() {
				dev_log!(
					"lifecycle",
					"error: [ApplicationRunTime] Failed to send effect result; receiver was dropped."
				);
			}
		};

		self.Scheduler.Submit(Task, Priority::Normal);

		match ResultReceiver.await {
			Ok(Result) => Result,

			Err(_) => {
				let Message = "Effect execution canceled; oneshot channel closed.".to_string();

				dev_log!("lifecycle", "error: {}", Message);

				Err(CommonError::IPCError { Description:Message }.into())
			},
		}
	}
}
