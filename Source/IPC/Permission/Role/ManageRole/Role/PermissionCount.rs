//! `Role::PermissionCount`

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Struct;
use crate::dev_log;

pub fn Fn(This:&Struct) -> usize { This.Permissions.len() }
