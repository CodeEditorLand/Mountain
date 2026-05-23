
//! Remove a value from the OS keychain.

use tonic::{Response, Status};
use CommonLibrary::Secret::SecretProvider::SecretProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{DeleteSecretRequest, Empty},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:DeleteSecretRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] delete_secret: key={}", Request.key);

	match Service.environment.DeleteSecret(String::new(), Request.key.clone()).await {
		Ok(()) => Ok(Response::new(Empty {})),

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] delete_secret failed key={}: {}",
				Request.key,
				Error
			);

			Err(Status::internal(format!("delete_secret: {}", Error)))
		},
	}
}
