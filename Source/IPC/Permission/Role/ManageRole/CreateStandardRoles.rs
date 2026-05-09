#![allow(non_snake_case)]

//! Build the standard `user` / `developer` / `admin` role
//! triple. Used by `Validator::Struct::InitializeDefaults` and
//! by tests.

use crate::{
	IPC::Permission::Role::ManageRole::{CreateAdminRole, CreateDeveloperRole, CreateUserRole, Role},
	dev_log,
};

pub fn Fn() -> Vec<Role::Struct> {

	dev_log!("ipc", "[ManageRole] Creating standard roles");

	vec![CreateUserRole::Fn(), CreateDeveloperRole::Fn(), CreateAdminRole::Fn()]
}
