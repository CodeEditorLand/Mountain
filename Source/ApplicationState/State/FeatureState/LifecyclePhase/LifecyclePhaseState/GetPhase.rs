//! `LifecyclePhaseState::GetPhase`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use tokio::sync::Notify;
use CommonLibrary::IPC::SkyEvent::SkyEvent;
use crate::{IPC::SkyEmit::Fn, dev_log};

pub fn Fn(This:&Struct) -> Phase { This.CurrentPhase.lock().ok().map(|Guard| *Guard).unwrap_or(1) }
