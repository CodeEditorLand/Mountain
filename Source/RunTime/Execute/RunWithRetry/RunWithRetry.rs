//! `RunWithRetry::RunWithRetry`

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};

use super::Struct;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn<TCapabilityProvider, TError, TOutput>(
	&self,

	Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,

	MaximumRetries:u32,

	InitialDelay:std::time::Duration,
) -> Result<TOutput, TError>
where
	TCapabilityProvider: ?Sized + Send + Sync + 'static,
	<Struct as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
		Requires<TCapabilityProvider>,
	TError: From<CommonError> + Send + Sync + 'static + std::fmt::Display,
	TOutput: Send + Sync + 'static, {
	let mut RetryCount = 0;

	let mut CurrentDelay = InitialDelay;

	while RetryCount <= MaximumRetries {
		match ApplicationRunTimeTrait::Run(self, Effect.clone()).await {
			Ok(Result) => return Ok(Result),

			Err(Error) => {
				if RetryCount == MaximumRetries {
					return Err(Error);
				}

				RetryCount += 1;

				dev_log!(
					"lifecycle",
					"warn: [ApplicationRunTime] Effect execution failed (attempt {}): {}. Retrying in {:?}...",
					RetryCount,
					Error,
					CurrentDelay
				);

				tokio::time::sleep(CurrentDelay).await;

				CurrentDelay *= 2;
			},
		}
	}

	Err(CommonError::Unknown { Description:format!("Effect execution failed after {} retries", MaximumRetries) }.into())
}
