//! `AirServiceProvider::UpdateConfiguration`

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

		section:String,

		updates:HashMap<String, String>,
	) -> Result<(), CommonError> {
		let RequestId = generate_request_id();

		dev_log!(
			"grpc",
			"[AirServiceProvider] update_configuration (request_id: {}, section: {})",
			request_id,
			section
		);

		This.Client.UpdateConfiguration(request_id, section, updates).await
	}
