//! `Error::ToTonicStatus`

use std::{
	net::AddrParseError,
	sync::{MutexGuard, PoisonError},
};

use http::uri::InvalidUri;
use thiserror::Error;

use super::Struct;

pub fn Fn(This:&Struct) -> tonic::Status {
	match self {
		Struct::RequestTimeout { .. } => tonic::Status::deadline_exceeded(This.to_string()),

		Struct::ClientNotConnected(_) | Struct::ConnectionFailed { .. } => tonic::Status::unavailable(This.to_string()),

		Struct::SerializationError(_) | Struct::InternalLockError(_) | Struct::InvalidState(_) => {
			tonic::Status::internal(This.to_string())
		},

		Struct::MessageTooLarge { .. } => tonic::Status::resource_exhausted(This.to_string()),

		Struct::InvalidMessageFormat(_) | Struct::InvalidUri(_) | Struct::AddressParseError(_) => {
			tonic::Status::invalid_argument(This.to_string())
		},

		Struct::RequestCanceled { .. } => tonic::Status::cancelled(This.to_string()),

		Struct::RPCError(msg) => tonic::Status::unknown(msg.clone()),

		Struct::ConnectionLost(_) => tonic::Status::aborted(This.to_string()),

		Struct::TonicTransportError(_) => tonic::Status::unavailable(This.to_string()),
	}
}
