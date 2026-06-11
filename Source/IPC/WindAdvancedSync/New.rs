//! Construct a `WindAdvancedSync` instance with empty document, UI-state,
//! real-time-update, and performance-stats containers.

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use crate::{
	IPC::{
		AdvancedFeatures::PerformanceStats::Struct as PerformanceStats,
		WindAdvancedSync::{
			DocumentSynchronization,
			GridLayout,
			LayoutState,
			RealTimeUpdateManager,
			SyncStatus,
			UIStateSynchronization,
			ViewState,
			WindAdvancedSync,
		},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub(crate) fn Fn(runtime:Arc<ApplicationRunTime>) -> WindAdvancedSync {
	WindAdvancedSync {
		runtime:runtime.clone(),

		document_sync:Arc::new(Mutex::new(DocumentSynchronization {
			synchronized_documents:HashMap::new(),
			pending_changes:HashMap::new(),
			last_sync_time:0,
			sync_status:SyncStatus {
				total_documents:0,
				synced_documents:0,
				conflicted_documents:0,
				offline_documents:0,
				last_sync_duration_ms:0,
			},
		})),

		ui_state_sync:Arc::new(Mutex::new(UIStateSynchronization {
			active_editor:None,
			cursor_positions:HashMap::new(),
			selection_ranges:HashMap::new(),
			view_state:ViewState {
				zoom_level:1.0,
				sidebar_visible:true,
				panel_visible:true,
				status_bar_visible:true,
			},
			theme:"default".to_string(),
			layout:LayoutState {
				editor_groups:Vec::new(),
				active_group:0,
				grid_layout:GridLayout { rows:1, columns:1, cell_width:100, cell_height:100 },
			},
		})),

		real_time_updates:Arc::new(Mutex::new(RealTimeUpdateManager {
			Updates:Vec::new(),
			Subscribers:HashMap::new(),
			UpdateQueue:Vec::new(),
			LastBroadcast:0,
		})),

		performance_stats:Arc::new(Mutex::new(PerformanceStats {
			total_messages_sent:0,
			total_messages_received:0,
			average_processing_time_ms:0.0,
			peak_message_rate:0,
			error_count:0,
			last_update:0,
			connection_uptime:0,
		})),
		// mountain_ipc: Arc::new(MountainIPC::new(runtime)), // Module doesn't exist
	}
}
