//! `Role::PermissionCount`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> usize { This.permissions.len() }
