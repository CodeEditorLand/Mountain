//! `ExtensionHostRegistry::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub fn Fn() -> Struct { Self { Hosts:Arc::new(RwLock::new(HashMap::new())) } }
