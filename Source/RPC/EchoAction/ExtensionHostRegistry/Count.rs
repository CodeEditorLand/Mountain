//! `ExtensionHostRegistry::Count`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub fn Fn(This:&Struct) -> usize { This.Hosts.read().await.len() }
