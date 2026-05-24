//! `Error::IsRecoverable`

use std::{
	net::AddrParseError,
	sync::{MutexGuard, PoisonError},
};

use http::uri::InvalidUri;
use thiserror::Error;

use super::Struct;

pub fn Fn(This:&Struct) -> bool {
	matches!(
		self,
		Struct::RequestTimeout { .. }
			| Struct::ConnectionFailed { .. }
			| Struct::ConnectionLost(_)
			| Struct::TonicTransportError(_)
	)
}
