//! `ServiceRegistry::Get`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&Struct, Name:&str) -> Option<&ServiceInfo::Struct> { This.Services.get(Name) }
