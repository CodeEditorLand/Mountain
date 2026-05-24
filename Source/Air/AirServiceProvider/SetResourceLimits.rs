//! `AirServiceProvider::SetResourceLimits`

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

		memory_limit_mb:u32,

		cpu_limit_percent:u32,

		disk_limit_mb:u32,
	) -> Result<(), CommonError> {
		let RequestId = generate_request_id();

		dev_log!("grpc", "[AirServiceProvider] set_resource_limits (request_id: {})", request_id);

		This.Client
			.SetResourceLimits(request_id, memory_limit_mb, cpu_limit_percent, disk_limit_mb)
			.await
	}
