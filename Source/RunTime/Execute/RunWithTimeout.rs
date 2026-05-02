#![allow(non_snake_case)]

//! Cancel an effect that exceeds a wall-clock budget. Wraps `Run` in
//! `tokio::time::timeout` and converts the elapsed-error into
//! `CommonError::Unknown`.

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

impl ApplicationRunTime {
	pub async fn RunWithTimeout<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
		Timeout:std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
			Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		tokio::time::timeout(Timeout, ApplicationRunTimeTrait::Run(self, Effect))
			.await
			.map_err(|_| {
				CommonError::Unknown { Description:format!("Effect execution timed out after {:?}", Timeout) }.into()
			})?
	}
}
