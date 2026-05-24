//! `NavigationHistoryState::GetStack`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Vec<String> { This.Stack.lock().ok().map(|G| G.clone()).unwrap_or_default() }
