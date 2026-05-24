//! `ServiceRegistry::New`

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

pub fn Fn() -> Struct {
		dev_log!("lifecycle", "[ServiceRegistry] Creating new ServiceRegistry");

		Self { services:Arc::new(RwLock::new(HashMap::new())), cert_manager:None }
	}
