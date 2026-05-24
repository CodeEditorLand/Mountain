//! `AirServiceProvider::GetMetrics`

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

pub fn Fn(This:&Struct, metric_type:Option<String>) -> Result<AirMetrics::Struct, CommonError> {
		let RequestId = generate_request_id();

		dev_log!("grpc", "[AirServiceProvider] get_metrics (request_id: {})", request_id);

		This.Client.GetMetrics(request_id, metric_type).await
	}
