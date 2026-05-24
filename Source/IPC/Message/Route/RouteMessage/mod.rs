pub mod New;
pub mod Register;
pub mod Remove;
pub mod RouteMessage;
pub mod GetChannels;
pub mod GetListenerCount;
pub mod ClearChannel;
pub mod ClearAll;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};
use crate::dev_log;

/// Message router for IPC channel-based message distribution
/// This router implements a publish-subscribe pattern where listeners can
/// register to receive messages on specific channels.
pub struct Struct {

	/// Map from channel names to their registered listeners
	listeners:Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>,
}
