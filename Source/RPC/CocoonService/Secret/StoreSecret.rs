#![allow(non_snake_case)]

//! Persist a value to the OS keychain.

use tonic::{Response, Status};

use CommonLibrary::Secret::SecretProvider::SecretProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, StoreSecretRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:StoreSecretRequest) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] store_secret: key={}", Request.key);

	match Service
		.environment
		.StoreSecret(String::new(), Request.key.clone(), Request.value)
		.await
	{

		Ok(()) => Ok(Response::new(Empty {})),

		Err(Error) => {

			dev_log!(
				"cocoon",

				"warn: [CocoonService] store_secret failed key={}: {}",

				Request.key,

				Error
			);

			Err(Status::internal(format!("store_secret: {}", Error)))
		},
	}
}
