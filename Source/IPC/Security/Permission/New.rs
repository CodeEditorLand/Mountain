//! `Permission::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(name:String, description:String, category:String) -> Struct { Self { name, description, category } }
