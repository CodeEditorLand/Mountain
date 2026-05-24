//! `AirServiceProvider::SearchFiles`

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

		query:String,

		path:String,

		max_results:u32,
	) -> Result<Vec<FileResult::Struct>, CommonError> {
		let RequestId = generate_request_id();

		dev_log!(
			"grpc",
			"[AirServiceProvider] search_files (request_id: {}, query: {})",
			request_id,
			query
		);

		This.Client.SearchFiles(request_id, query, path, max_results).await
	}
