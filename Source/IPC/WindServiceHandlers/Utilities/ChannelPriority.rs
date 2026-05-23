
//! Maps a wire command string → Echo scheduler lane via the Common
//! `Channel` registry's `Priority()` accessor. Unknown commands fall
//! back to `Priority::Normal` so unclassified callers don't starve the
//! high-priority queue.

use Echo::Task::Priority::Priority as EchoPriority;

pub fn Fn(Command:&str) -> EchoPriority {
	use std::str::FromStr;

	match CommonLibrary::IPC::Channel::Channel::from_str(Command) {
		Ok(Channel) => {
			match Channel.Priority() {
				CommonLibrary::IPC::Channel::ChannelPriority::High => EchoPriority::High,

				CommonLibrary::IPC::Channel::ChannelPriority::Normal => EchoPriority::Normal,

				CommonLibrary::IPC::Channel::ChannelPriority::Low => EchoPriority::Low,
			}
		},

		Err(_) => EchoPriority::Normal,
	}
}
