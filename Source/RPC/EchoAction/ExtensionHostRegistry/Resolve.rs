//! `ExtensionHostRegistry::Resolve`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub fn Fn(This:&Struct, ExtensionIdentifier:&str) -> Option<String> {
		This.Hosts.read().await.get(ExtensionIdentifier).cloned()
	}
