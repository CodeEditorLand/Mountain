
//! Wrapper for an asynchronous Air download stream. Adapts the tonic
//! streaming API into a `next().await` iterator that yields
//! `DownloadStreamChunk::Struct` items. Cfg-gated on `AirIntegration`
//! because the inner type lives in `AirLibrary::Vine::Generated::air`.

#[cfg(feature = "AirIntegration")]
use CommonLibrary::Error::CommonError::CommonError;

#[cfg(feature = "AirIntegration")]
use crate::{Air::AirClient::DownloadStreamChunk, dev_log};

#[cfg(feature = "AirIntegration")]
pub struct Struct {
	inner:tonic::codec::Streaming<AirLibrary::Vine::Generated::air::DownloadStreamResponse>,
}

#[cfg(feature = "AirIntegration")]
impl Struct {
	pub fn new(Stream:tonic::codec::Streaming<AirLibrary::Vine::Generated::air::DownloadStreamResponse>) -> Self {
		Self { inner:Stream }
	}

	/// Returns the next chunk from the stream. `None` when the stream ends.
	pub async fn next(&mut self) -> Option<Result<DownloadStreamChunk::Struct, CommonError>> {
		match futures_util::stream::StreamExt::next(&mut self.inner).await {
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
}
