//! `SecurityContext::HasRole`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct, role:&str) -> bool { This.roles.iter().any(|r| r == role) }
