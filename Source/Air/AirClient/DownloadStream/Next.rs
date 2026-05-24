//! `DownloadStream::Next`

use super::Struct;
use CommonLibrary::Error::CommonError::CommonError;
use crate::{Air::AirClient::DownloadStreamChunk, dev_log};

pub fn Fn(This:&mut Struct) -> Option<Result<DownloadStreamChunk::Struct, CommonError>> {
		match futures_util::stream::StreamExt::next(&mut This.inner).await {
			Some(Ok(Response)) => {
				Some(Ok(DownloadStreamChunk::Struct {
					data:Response.chunk,
					total_size:Response.total_size,
					downloaded:Response.downloaded,
					completed:Response.completed,
					error:Response.error,
				}))
			},

			Some(Err(Error)) => {
				dev_log!("grpc", "error: [DownloadStream] Stream error: {}", Error);

				Some(Err(CommonError::IPCError { Description:format!("Stream error: {}", Error) }))
			},

			None => None,
		}
	}
