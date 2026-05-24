//! `ServiceRegistry::GetPort`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use http::{Request as HttpRequest, Response as HttpResponse, header};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> u16 {
		if This.use_tls {
			This.tls_port.unwrap_or_else(|| This.port + 1000)
		} else {
			This.port
		}
	}
