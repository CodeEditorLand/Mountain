//! `RunWithTimeout::RunWithTimeout`

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};

use super::Struct;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub fn Fn<TCapabilityProvider, TError, TOutput>(
	&self,

	Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,

	Timeout:std::time::Duration,
) -> Result<TOutput, TError>
where
	TCapabilityProvider: ?Sized + Send + Sync + 'static,
	<Struct as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
		Requires<TCapabilityProvider>,
	TError: From<CommonError> + Send + Sync + 'static,
	TOutput: Send + Sync + 'static, {
	tokio::time::timeout(Timeout, ApplicationRunTimeTrait::Run(self, Effect))
		.await
		.map_err(|_| {
			CommonError::Unknown { Description:format!("Effect execution timed out after {:?}", Timeout) }.into()
		})?
}
