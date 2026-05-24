//! `Validator::CreateSecurityContext`

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::sync::RwLock;

use super::Struct;
use crate::{
	IPC::Permission::{
		Role::ManageRole::{Permission::Struct as Permission, Role::Struct as Role},
		Validate::ValidatePermission::SecurityContext::Struct as SecurityContext,
	},
	dev_log,
};

pub fn Fn(UserId:String, Roles:Vec<String>, IpAddress:String, DirectPermissions:Vec<String>) -> SecurityContext {
	let ValidRoles = if Roles.is_empty() { vec!["user".to_string()] } else { Roles };

	let ValidIpAddress = if IpAddress.is_empty() { "127.0.0.1".to_string() } else { IpAddress };

	SecurityContext {
		UserId,

		Roles:ValidRoles,

		Permissions:DirectPermissions,

		IpAddress:ValidIpAddress,

		Timestamp:SystemTime::now(),
	}
}
