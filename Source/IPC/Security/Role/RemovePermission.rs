//! `Role::RemovePermission`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, permission:&str) { This.permissions.retain(|p| p != permission); }
