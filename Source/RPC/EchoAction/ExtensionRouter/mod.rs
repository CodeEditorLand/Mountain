pub mod New;
pub mod HostFor;

use std::sync::Arc;
use crate::RPC::EchoAction::ExtensionHostRegistry;

pub struct Struct {
	Registry:Arc<ExtensionHostRegistry::Struct>,
}
