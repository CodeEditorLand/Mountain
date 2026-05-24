//! `DownloadStream::New`

use super::Struct;
use CommonLibrary::Error::CommonError::CommonError;
use crate::{Air::AirClient::DownloadStreamChunk, dev_log};

pub fn Fn(Stream:tonic::codec::Streaming<AirLibrary::Vine::Generated::air::DownloadStreamResponse>) -> Struct {
		Self { inner:Stream }
	}
