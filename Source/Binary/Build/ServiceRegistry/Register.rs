//! `ServiceRegistry::Register`

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

pub fn Fn(This:&Struct, name:String, port:u16, health_check_path:Option<String>) {
		This.RegisterWithOptions(name, port, None, false, health_check_path);
	}
