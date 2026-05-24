//! `ServiceRegistry::WithTls`

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

pub fn Fn(
		cert_manager:std::sync::Arc<std::sync::Mutex<super::CertificateManager::CertificateManager>>,
	) -> Struct {
		dev_log!("lifecycle", "[ServiceRegistry] Creating new ServiceRegistry with TLS support");

		Self { services:Arc::new(RwLock::new(HashMap::new())), cert_manager:Some(cert_manager) }
	}
