#![allow(non_snake_case)]
//! `task` provider-unregistration atom.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterTaskProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "task");
}
