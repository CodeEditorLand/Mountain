//! `AirServiceProvider::IndexFiles`

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

		path:String,

		patterns:Vec<String>,

		exclude_patterns:Vec<String>,

		max_depth:u32,
	) -> Result<IndexInfo::Struct, CommonError> {
		let RequestId = generate_request_id();

		dev_log!(
			"grpc",
			"[AirServiceProvider] index_files (request_id: {}, path: {})",
			request_id,
			path
		);

		This.Client
			.IndexFiles(request_id, path, patterns, exclude_patterns, max_depth)
			.await
	}
