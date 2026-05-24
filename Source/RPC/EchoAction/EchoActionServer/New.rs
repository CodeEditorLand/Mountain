//! `EchoActionServer::New`

use super::Struct;
use std::sync::Arc;
use Echo::{Scheduler::Scheduler::Scheduler, Task::Priority::Priority as EchoPriority};
use tokio::sync::oneshot;
use crate::RPC::EchoAction::{ExtensionHostRegistry, ResolveMethodPriority};

pub fn Fn() -> Struct { Self { Registry:Arc::new(ExtensionHostRegistry::Struct::new()) } }
