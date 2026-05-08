#![allow(non_snake_case)]

//! Retry a failing effect with exponential back-off, doubling the inter-
//! attempt delay after each failure to avoid overwhelming the recovering
//! system.

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn RunWithRetry<TCapabilityProvider, TError, TOutput>(
		&self,

		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,

		MaximumRetries:u32,

		InitialDelay:std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
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

		Err(
			CommonError::Unknown { Description:format!("Effect execution failed after {} retries", MaximumRetries) }
				.into(),
		)
	}
}
