//! `LifecyclePhaseState::SetPhase`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use tokio::sync::Notify;
use CommonLibrary::IPC::SkyEvent::SkyEvent;
use crate::{IPC::SkyEmit::Fn, dev_log};

pub fn Fn(This:&Struct, NewPhase:Phase) {
		if let Ok(mut Guard) = This.CurrentPhase.lock() {
			if NewPhase > *Guard {
				dev_log!("lifecycle", "[LifecyclePhaseState] Phase advanced: {} → {}", *Guard, NewPhase);

				*Guard = NewPhase;

				// Wake all `LifecycleWhenPhase` waiters immediately.
				This.PhaseNotify.notify_waiters();
			}
		}
	}
