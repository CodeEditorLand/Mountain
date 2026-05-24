//! `Role::HasPermission`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct, permission:&str) -> bool { This.permissions.iter().any(|p| p == permission) }
