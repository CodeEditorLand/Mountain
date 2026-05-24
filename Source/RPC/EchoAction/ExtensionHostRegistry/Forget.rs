//! `ExtensionHostRegistry::Forget`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub fn Fn(This:&Struct, ExtensionIdentifier:&str) { This.Hosts.write().await.remove(ExtensionIdentifier); }
