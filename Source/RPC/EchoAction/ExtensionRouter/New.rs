//! `ExtensionRouter::New`

use super::Struct;
use std::sync::Arc;
use crate::RPC::EchoAction::ExtensionHostRegistry;

pub fn Fn(Registry:Arc<ExtensionHostRegistry::Struct>) -> Struct { Self { Registry } }
