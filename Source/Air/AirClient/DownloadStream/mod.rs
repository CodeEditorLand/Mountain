pub mod New;
pub mod Next;

use CommonLibrary::Error::CommonError::CommonError;
use crate::{Air::AirClient::DownloadStreamChunk, dev_log};

#[cfg(feature = "AirIntegration")]
pub struct Struct {
	inner:tonic::codec::Streaming<AirLibrary::Vine::Generated::air::DownloadStreamResponse>,
}

#[cfg(not(feature = "AirIntegration"))]
pub struct Struct;
