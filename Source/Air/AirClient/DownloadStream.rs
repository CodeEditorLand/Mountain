//! Wrapper for an asynchronous Air download stream. Adapts the tonic
//! streaming API into a `next().await` iterator that yields
//! `DownloadStreamChunk::Struct` items.

/// Type alias for Struct.
pub type Struct = ::AirLibrary::Client::AirClient::DownloadStream::Struct;
