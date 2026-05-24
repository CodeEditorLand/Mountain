//! `ServiceRegistry::GetMut`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&mut Struct, Name:&str) -> Option<&mut ServiceInfo::Struct> { This.Services.get_mut(Name) }
