#![allow(non_snake_case)]

//! `Permission::Struct` - RBAC permission descriptor.
//! `category.action` name shape (validated by `Validate`),
//! human description, category bucket, and an `IsSensitive`
//! flag that drives elevated audit logging in the
//! `LogEvent` module.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub Name:String,

	pub Description:String,

	pub Category:String,

	pub IsSensitive:bool,
}

impl Struct {

	pub fn New(Name:String, Description:String, Category:String) -> Self {

		Self { Name, Description, Category, IsSensitive:false }
	}

	pub fn NewSensitive(Name:String, Description:String, Category:String) -> Self {

		Self { Name, Description, Category, IsSensitive:true }
	}

	pub fn SetSensitive(mut self) -> Self {

		self.IsSensitive = true;

		self
	}

	pub fn GetAction(&self) -> String { self.Name.rsplit('.').next().unwrap_or("unknown").to_string() }

	pub fn GetCategory(&self) -> String {

		if let Some(pos) = self.Name.rfind('.') {

			self.Name[..pos].to_string()
		} else {

			"unknown".to_string()
		}
	}

	pub fn Validate(&self) -> Result<(), String> {

		if self.Name.is_empty() {

			return Err("Permission name cannot be empty".to_string());
		}

		if self.Name.contains(|c:char| c.is_whitespace()) {

			return Err("Permission name cannot contain whitespace".to_string());
		}

		if !self.Name.contains('.') {

			return Err("Permission name must contain a dot separating category and action".to_string());
		}

		if self.Description.is_empty() {

			return Err("Permission description cannot be empty".to_string());
		}

		if self.Category.is_empty() {

			return Err("Permission category cannot be empty".to_string());
		}

		Ok(())
	}
}
