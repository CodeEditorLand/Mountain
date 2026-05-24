//! `AirServiceProvider::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use CommonLibrary::Error::CommonError::CommonError;
use super::AirClient::{Struct,
	AirMetrics,
	AirStatus,
	DEFAULT_AIR_SERVER_ADDRESS,
	DownloadStream,
	DownloadStreamChunk,
	ExtendedFileInfo,
	FileInfo,
	FileResult,
	IndexInfo,
	ResourceUsage,
	UpdateInfo,
};
use crate::{Air::AirServiceProvider::GenerateRequestID::Fn as generate_request_id, dev_log};

pub fn Fn(address:String) -> Result<Self, CommonError> {
		dev_log!("grpc", "[AirServiceProvider] Creating AirServiceProvider at: {}", address);

		let client = AirClient::new(&address).await?;

		dev_log!("grpc", "[AirServiceProvider] AirServiceProvider created successfully");

		Ok(Self { client:Arc::new(client) })
	}
