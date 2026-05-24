//! `Permission::NewSensitive`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(Name:String, Description:String, Category:String) -> Struct {
	Self { Name, Description, Category, IsSensitive:true }
}
