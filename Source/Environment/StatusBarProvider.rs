//! # StatusBarProvider (Environment)
//!
//! Implements the `StatusBarProvider` trait for `MountainEnvironment`, managing
//! the creation, update, and removal of status bar items at the bottom of the
//! application window.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Status Bar Item Management
//! - Create status bar items with unique identifiers
//! - Update item text, tooltip, and visibility
//! - Remove items when no longer needed
//! - Manage item lifecycle from creation to disposal
//!
//! ### 2. Status Bar Organization
//! - Support left and right alignment groups
//! - Implement priority ordering within alignment groups
//! - Handle item visibility and spacing
//! - Manage item ordering and z-index
//!
//! ### 3. Dynamic Content
//! - Support text updates in real-time (e.g., git branch, encoding)
//! - Handle command association on click
//! - Support color and style customization
//! - Enable tooltip resolution via callbacks
//!
//! ### 4. Extension Integration
//! - Allow extensions to contribute status bar items
//! - Route status bar commands to extension handlers
//! - Manage extension-provided item lifecycle
//! - Coordinate with Cocoon sidecar for extension UI
//!
//! ## ARCHITECTURAL ROLE
//!
//! StatusBarProvider is the **status bar orchestrator**:
//!
//! ```text
//! Provider ──► Create/Update ──► Status Bar UI (Sky)
//!       │                            │
//!       └─► Command ──► DispatchLogic ──► Handler
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: UI capability provider
//! - Implements `CommonLibrary::StatusBar::StatusBarProvider` trait
//! - Accessible via `Environment.Require<dyn StatusBarProvider>()`
//!
//! ### Status Bar State
//! - `ApplicationState.ActiveStatusBarItems`: HashMap<String, StatusBarEntryDTO>
//!   - Key: Status bar item ID (unique)
//!   - Value: StatusBarEntryDTO with text, tooltip, alignment, priority, etc.
//!
//! ### UI Representation
//! - Items displayed horizontally at bottom of window
//! - Left-aligned items: From left to right by descending priority
//! - Right-aligned items: From right to left by descending priority
//! - Items can be shown/hidden dynamically
//!
//! ### Dependencies
//! - `ApplicationState`: Status bar item storage
//! - `IPCProvider`: To send updates to Sky frontend
//! - `CommandExecutor`: For status bar item command execution
//! - `Log`: Status bar change logging
//!
//! ### Dependents
//! - Extensions: Create status bar items via `createStatusBarEntry`
//! - Built-in commands: Show encoding, line endings, git branch, etc.
//! - `Binary::Main`: Initialize status bar during startup
//! - `DispatchLogic`: Route status bar command invocations
//!
//! ## STATUS BAR ITEM PROPERTIES
//!
//! A `StatusBarEntryDTO` includes:
//! - `ID`: Unique identifier for the item
//! - `Text`: Display text (supports `$` variables like `$(line)`)
//! - `Tooltip`: Hover tooltip (static or dynamic via callback)
//! - `Alignment`: Left or Right
//! - `Priority`: Ordering within alignment group (higher = more left)
//! - `Command`: Command ID to execute on click
//! - `IsVisible`: Show/hide flag
//! - `Color`: Optional text color override
//! - `BackgroundColor`: Optional background color
//!
//! ## PRIORITY SYSTEM
//!
//! Priority determines horizontal ordering within an alignment group:
//! - **Higher priority** items appear **to the left** (for left-aligned)
//! - **Higher priority** items appear **to the right** (for right-aligned)
//! - Default priorities: 0-100 range
//! - Extension items typically use priorities 0-50
//! - Built-in items use priorities 50-100
//!
//! Example left-aligned ordering (high to low):
//! ```
//! [Encoding: UTF-8] [Line: LF] [Git: main*] [Branch: main]
//!   Priority 100     80          70           60
//! ```
//!
//! ## DYNAMIC TOOLTIP RESOLUTION
//!
//! Tooltips can be static strings or dynamic via a sidecar callback:
//! 1. Item created with `tooltip: "Loading..."` and `resolve_tooltip: true`
//! 2. When user hovers, Sky requests tooltip from Mountain
//! 3. Mountain calls extension's `onDidHover` callback (via IPC)
//! 4. Extension returns dynamic tooltip content
//! 5. Sky updates tooltip display
//!
//! ## ERROR HANDLING
//!
//! - Duplicate ID: `CommonError::InvalidArgument` (or replace existing)
//! - Invalid alignment: Default to left
//! - Missing command: Log warning, item still created
//! - IPC failures: Log error, item may not update in UI
//!
//! ## PERFORMANCE
//!
//! - Status bar updates are batched and sent via IPC efficiently
//! - Item lookup by ID is O(1) via HashMap
//! - Priority ordering uses sorting (acceptable for small item counts)
//! - Dynamic tooltips incur IPC round-trip (cache if频繁)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/api/common/extHostStatusBar.ts` - Extension API
//! - `vs/platform/statusbar/common/statusbar.ts` - Status bar service
//! - `vs/workbench/services/statusbar/common/statusbar.ts` - Implementation
//!
//! ## TODO
//!
//! - [ ] Implement priority ordering validation and collision detection
//! - [ ] Add status bar item animation support (fade, flash)
//! - [ ] Support status bar item color themes integration
//! - [ ] Implement status bar item grouping and separators
//! - [ ] Add status bar item context menu on right-click
//! - [ ] Support status bar widget contributions from extensions
//! - [ ] Implement status bar item compact mode (smaller text)
//! - [ ] Add status bar item accessibility (ARIA labels)
//! - [ ] Implement status bar item hover actions (click + hover)
//! - [ ] Support status bar item configuration persistence
//! - [ ] Add status bar item command arguments customization
//! - [ ] Implement status bar item progress indicator (spinner)
//! - [ ] Support status bar item badge (notification count)
//! - [ ] Add status bar item drag-reordering (user customization)
//!
//! ## MODULE CONTENTS
//!
//! - [`StatusBarProvider`]: Main struct implementing the trait
//! - Item creation, update, and removal methods
//! - Priority ordering and alignment management
//! - Dynamic tooltip resolution
//! - Command invocation from status bar clicks
//! - State persistence and restoration

//   - Implement status bar priority ordering system
//   - Add status bar alignment (left/right) support
//   - Implement status bar item visibility toggle
//   - Support status bar item compact mode
//   - Add status bar item background color support
//   - Implement status bar item grouping
//   - Support status bar item command registration
//   - Add status bar item accessibility (ARIA labels)
//   - Implement status bar item hover actions
//   - Support status bar widget contribution points
//   - Add status bar item animation support
//   - Implement status bar item context menu
//   - Add status bar configuration persistence
//
// Inspired by VSCode's status bar service which:
// - Uses IStatusbarEntryPriority for item ordering
// - Supports StatusbarAlignment (Left/Right)
// - Provides dynamic tooltip resolution
// - Manages entry visibility overrides
// - Supports status bar item compact mode
// - Handles status bar item grouping
//
// ## Status Bar Priority System
//
// The priority determines the order of items within their alignment group:
// - Higher priority values appear before lower priority values
// - Left alignment: Items arranged from left to right by descending priority
// - Right alignment: Items arranged from right to left by descending priority
// - Default priority is 0 for items without explicit priority
// - Primary items typically use priority 100-1000
// - Secondary items typically use priority 10-99
//
// ## Status Bar Item Types
//
// 1. **Persistent Items**: Long-lived items (e.g., branch indicator, language
//    indicator)
// 2. **Transient Messages**: Temporary notifications that auto-dismiss
// 3. **Dynamic Items**: Items with computed values (e.g., error count,
//    position)

//! # StatusBarProvider Implementation
//!
//! Implements the `StatusBarProvider` trait for the `MountainEnvironment`. This
//! provider handles creating, updating, and removing status bar items, and
//! orchestrates communication between the `Cocoon` sidecar and the `Sky`
//! frontend.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Creates a new status bar entry or updates an existing one.
	async fn SetStatusBarEntry(&self, Entry:StatusBarEntryDTO) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Setting entry: {}", Entry.EntryIdentifier);

		let mut ItemsGuard = self
			.ApplicationState
			.ActiveStatusBarItems
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		ItemsGuard.insert(Entry.EntryIdentifier.clone(), Entry.clone());

		drop(ItemsGuard);

		self.ApplicationHandle
			.emit("sky://statusbar/set-entry", Entry)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Removes a status bar item from the UI.
	async fn DisposeStatusBarEntry(&self, EntryIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing entry: {}", EntryIdentifier);

		self.ApplicationState
			.ActiveStatusBarItems
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&EntryIdentifier);

		self.ApplicationHandle
			.emit("sky://statusbar/dispose-entry", json!({ "EntryIdentifier": EntryIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Shows a temporary message in the status bar.
	async fn SetStatusBarMessage(&self, MessageIdentifier:String, Text:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Setting status message '{}': {}", MessageIdentifier, Text);

		self.ApplicationHandle
			.emit("sky://statusbar/set-message", json!({ "id": MessageIdentifier, "text": Text }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Disposes of a temporary status bar message.
	async fn DisposeStatusBarMessage(&self, MessageIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing status message '{}'", MessageIdentifier);

		self.ApplicationHandle
			.emit("sky://statusbar/dispose-message", json!({ "id": MessageIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Resolves a dynamic tooltip by making a reverse call to the extension
	/// host.
	async fn ProvideTooltip(&self, EntryIdentifier:String) -> Result<Option<Value>, CommonError> {
		info!("[StatusBarProvider] Providing dynamic tooltip for entry: {}", EntryIdentifier);

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		// This is a "reverse" call, where the host needs data from the sidecar.
		let RPCMethod = format!("{}$ProvideStatusbarTooltip", ProxyTarget::ExtHostStatusBar.GetTargetPrefix());

		let RPCResponse = IPCProvider
			.SendRequestToSideCar("cocoon-main".to_string(), RPCMethod, json!([EntryIdentifier]), 5000)
			.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(RPCResponse).unwrap_or(None))
	}
}
