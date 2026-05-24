//! `AirServiceProvider::GetFileInfo`

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

pub fn Fn(This:&Struct, path:String) -> Result<ExtendedFileInfo::Struct, CommonError> {
		let RequestId = generate_request_id();

		dev_log!(
			"grpc",
			"[AirServiceProvider] get_file_info (request_id: {}, path: {})",
			request_id,
			path
		);

		This.Client.GetFileInfo(request_id, path).await
	}
