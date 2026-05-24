pub mod New;
pub mod Registry;
pub mod Dispatch;

use std::sync::Arc;
use Echo::{Scheduler::Scheduler::Scheduler, Task::Priority::Priority as EchoPriority};
use tokio::sync::oneshot;
use crate::RPC::EchoAction::{ExtensionHostRegistry, ResolveMethodPriority};

#[derive(Clone)]
pub struct Struct {
	Registry:Arc<ExtensionHostRegistry::Struct>,
}
