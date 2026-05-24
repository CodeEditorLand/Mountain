//! `Struct::GetNextSourceControlManagementProviderHandle`

use super::Struct;
use super::{
	Debug::DebugState::DebugState,
	Decorations::DecorationsState::DecorationsState,
	Diagnostics::DiagnosticsState::DiagnosticsState,
	Documents::DocumentState::DocumentState,
	Keybindings::KeybindingState::KeybindingState,
	LifecyclePhase::LifecyclePhaseState::LifecyclePhaseState,
	Markers::MarkerState::MarkerState,
	NavigationHistory::NavigationHistoryState::NavigationHistoryState,
	OutputChannels::OutputChannelState::OutputChannelState,
	Terminals::TerminalState::TerminalState,
	TreeViews::TreeViewState::TreeViewState,
	Webviews::WebviewState::WebviewState,
	WorkingCopy::WorkingCopyState::WorkingCopyState,
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> u32 {
		This.Markers.GetNextSourceControlManagementProviderHandle()
	}
