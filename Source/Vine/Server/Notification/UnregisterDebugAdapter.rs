#![allow(non_snake_case)]
//! `debug_adapter` provider-unregistration atom.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterDebugAdapter(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "debug_adapter");
}
