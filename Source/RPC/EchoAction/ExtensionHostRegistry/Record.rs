//! `ExtensionHostRegistry::Record`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub fn Fn(This:&Struct, ExtensionIdentifier:String, HostIdentifier:String) {
		This.Hosts.write().await.insert(ExtensionIdentifier, HostIdentifier);
	}
