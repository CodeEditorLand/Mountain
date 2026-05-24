//! `RouteMessage::New`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

pub fn Fn() -> Struct { Self { listeners:Arc::new(Mutex::new(HashMap::new())) } }
