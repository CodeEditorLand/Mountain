
//! `Role::Struct` - RBAC role descriptor. Builder methods
//! deduplicate permissions on insert, expose
//! `HasPermission` / `PermissionCount` lookups, and
//! `Validate` enforces the `category.action` permission name
//! shape so misconfigured roles fail loudly at registration.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::dev_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Name:String,

	pub Permissions:Vec<String>,

	pub Description:String,

	pub ParentRole:Option<String>,

	pub Priority:u32,
}

impl Struct {
	pub fn New(Name:String, Permissions:Vec<String>, Description:String) -> Self {
		let UniquePermissions:Vec<String> = Permissions.into_iter().collect::<HashSet<String>>().into_iter().collect();

		Self { Name, Permissions:UniquePermissions, Description, ParentRole:None, Priority:0 }
	}

	pub fn NewWithParent(
		Name:String,

		Permissions:Vec<String>,

		Description:String,

		ParentRole:String,

		Priority:u32,
	) -> Self {
		let UniquePermissions:Vec<String> = Permissions.into_iter().collect::<HashSet<String>>().into_iter().collect();

		Self {
			Name,

			Permissions:UniquePermissions,

			Description,

			ParentRole:Some(ParentRole),

			Priority,
		}
	}

	pub fn AddPermission(mut self, Permission:String) -> Self {
		if !self.Permissions.contains(&Permission) {
			self.Permissions.push(Permission.clone());

			dev_log!("ipc", "[Role] Added permission '{}' to role '{}'", Permission, self.Name);
		}

		self
	}

	pub fn AddPermissions(mut self, Permissions:impl IntoIterator<Item = String>) -> Self {
		for Permission in Permissions {
			if !self.Permissions.contains(&Permission) {
				self.Permissions.push(Permission.clone());

				dev_log!("ipc", "[Role] Added permission '{}' to role '{}'", Permission, self.Name);
			}
		}

		self
	}

	pub fn HasPermission(&self, Permission:&str) -> bool { self.Permissions.contains(&Permission.to_string()) }

	pub fn PermissionCount(&self) -> usize { self.Permissions.len() }

	pub fn Validate(&self) -> Result<(), String> {
		if self.Name.is_empty() {
			return Err("Role name cannot be empty".to_string());
		}

		if self.Name.contains(|c:char| c.is_whitespace()) {
			return Err("Role name cannot contain whitespace".to_string());
		}

		if self.Description.is_empty() {
			return Err("Role description cannot be empty".to_string());
		}

		for Permission in &self.Permissions {
			if Permission.is_empty() {
				return Err("Permission name cannot be empty".to_string());
			}

			if !Permission.contains('.') {
				return Err(format!(
					"Permission '{}' must contain a dot separating category and action",
					Permission
				));
			}

			if Permission.contains(|c:char| c.is_whitespace()) {
				return Err(format!("Permission '{}' cannot contain whitespace", Permission));
			}
		}

		Ok(())
	}
}
