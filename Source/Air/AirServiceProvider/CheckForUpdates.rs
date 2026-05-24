//! `AirServiceProvider::CheckForUpdates`

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

		current_version:String,

		channel:String,
	) -> Result<Option<UpdateInfo::Struct>, CommonError> {
		let RequestId = generate_request_id();

		dev_log!("grpc", "[AirServiceProvider] check_for_updates (request_id: {})", request_id);

		let info = This.Client.CheckForUpdates(request_id, current_version, channel).await?;

		if info.update_available { Ok(Some(info)) } else { Ok(None) }
	}
