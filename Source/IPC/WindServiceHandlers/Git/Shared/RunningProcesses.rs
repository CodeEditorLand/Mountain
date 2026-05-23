
//! Returns the global PID registry for cancel-by-OperationId.

use std::{collections::HashMap, sync::Mutex};

pub fn Fn() -> &'static Mutex<HashMap<String, u32>> { super::running_processes() }
