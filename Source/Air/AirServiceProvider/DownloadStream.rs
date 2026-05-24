//! `AirServiceProvider::DownloadStream`

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

pub fn Fn(
		&self,

		url:String,

		headers:HashMap<String, String>,
	) -> Result<DownloadStream::Struct, CommonError> {
		let RequestId = generate_request_id();

		dev_log!(
			"grpc",
			"[AirServiceProvider] download_stream (request_id: {}, url: {})",
			request_id,
			url
		);

		This.Client.DownloadStream(request_id, url, headers).await
	}
