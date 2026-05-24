//! `Permission::IsInCategory`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct, category:&str) -> bool { This.category == category }
