//! Read a value from the OS keychain. The gRPC proto carries only `key`;
//! the app name is used as the keyring service scope.
use tonic::{Response, Status};
use CommonLibrary::Secret::SecretProvider::SecretProvider;
use ::Vine::Generated::{GetSecretRequest, GetSecretResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:GetSecretRequest) -> Result<Response<GetSecretResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] get_secret: key={}", Request.key);

	match Service.environment.GetSecret(String::new(), Request.key.clone()).await {
		Ok(Some(Value)) => Ok(Response::new(GetSecretResponse { value:Value })),

		Ok(None) => Ok(Response::new(GetSecretResponse { value:String::new() })),

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] get_secret failed key={}: {}",
				Request.key,
				Error
			);

			Err(Status::internal(format!("get_secret: {}", Error)))
		},
	}
}
