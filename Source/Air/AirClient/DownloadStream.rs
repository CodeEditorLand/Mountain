//! Wrapper for an asynchronous Air download stream. Adapts the tonic
//! streaming API into a `next().await` iterator that yields
//! `DownloadStreamChunk::Struct` items.

pub type Struct = ::Air::Client::AirClient::DownloadStream::Struct;
