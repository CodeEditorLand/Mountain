//! `ExtensionRouter::HostFor`

use super::Struct;
use std::sync::Arc;
use crate::RPC::EchoAction::ExtensionHostRegistry;

pub fn Fn(This:&Struct, ExtensionIdentifier:&str) -> Option<String> {
		This.Registry.Resolve(ExtensionIdentifier).await
	}
