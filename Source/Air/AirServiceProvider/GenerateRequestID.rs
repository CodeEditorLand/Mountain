//! Generate a fresh UUID-v4 (simple form) for use as an Air request id.
//! Each Air RPC carries one of these so Mountain can correlate replies
//! with the originating call across log lines + traces.

pub fn Fn() -> String { ::AirLibrary::Client::AirServiceProvider::GenerateRequestID::Fn() }
