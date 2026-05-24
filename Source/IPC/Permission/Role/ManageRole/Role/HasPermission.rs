//! `Role::HasPermission`

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Struct;
use crate::dev_log;

pub fn Fn(This:&Struct, Permission:&str) -> bool { This.Permissions.contains(&Permission.to_string()) }
