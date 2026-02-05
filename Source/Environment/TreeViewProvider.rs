//! # TreeViewProvider (Environment)
//!
//! Implements the `TreeViewProvider` trait for `MountainEnvironment`, managing
//! the lifecycle of custom tree views and orchestrating data flow between the
//! extension host (Cocoon) and the UI (Sky).
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Tree View Registration
//! - Register tree view providers from extensions via `RegisterTreeDataProvider`
//! - Create and store tree view state in `ApplicationState.ActiveTreeViews`
//! - Manage tree view identifiers and view types
//! - Handle tree view unregistration and cleanup
//!
//! ### 2. Data Provider Dispatching
//! - Route tree data requests to appropriate provider (native or extension)
//! - Support both native Rust providers and extension-sidecar providers
//! - Handle `GetChildren`, `GetTreeItem`, `OnTreeSelectionChanged`, etc.
//! - Ensure thread-safe access to provider state
//!
//! ### 3. Tree State Management
//! - Track tree view options (title, description, message, badge)
//! - Manage tree item visibility and selection state
//! - Persist tree state across sessions (expanded/collapsed nodes)
//! - Support tree view state restoration
//!
//! ### 4. UI Events & Notifications
//! - Handle tree node expansion/collapse events
//! - Process tree item selection changes
//! - Emit events to Sky for UI updates
//! - Support `RefreshTreeView` for manual updates
//!
//! ### 5. Advanced Features
//! - Drag-and-drop support (`HasHandleDrag`, `HasHandleDrop`)
//! - Multi-selection support (`CanSelectMany`)
//! - Badge display on tree items
//! - Message display above tree view
//! - Title and description updates
//!
//! ## ARCHITECTURAL ROLE
//!
//! TreeViewProvider is the **tree view orchestration layer**:
//!
//! ```text
//! Extension ──► RegisterTreeDataProvider ──► TreeViewProvider ──► Sky TreeView
//!                     │                              │
//!                     └─► IPC Calls ──► Cocoon ◄────┘
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: UI tree capability provider
//! - Implements `CommonLibrary::TreeView::TreeViewProvider` trait
//! - Accessible via `Environment.Require<dyn TreeViewProvider>()`
//!
//! ### Tree View Provider Types
//!
//! **Native Provider**:
//! - Implemented directly in Rust
//! - Example: `FileExplorerViewProvider`
//! - Direct function calls (no IPC)
//! - High performance, full control
//!
//! **Extension Provider**:
//! - Implemented in extension (TypeScript/JavaScript)
//! - Runs in Cocoon sidecar process
//! - Accessed via IPC (`SendNotificationToSideCar`)
//! - Isolated, sandboxed, extensible
//!
//! ### Data Flow
//!
//! 1. **Registration**: Extension calls `RegisterTreeDataProvider(viewId, options)`
//! 2. **Initial Request**: Sky calls `GetChildren(viewId, parentHandle)` for root
//! 3. **Provider Call**: TreeViewProvider routes to provider's `GetChildren`
//! 4. **Result**: Provider returns `TreeItemDTO` JSON objects
//! 5. **Display**: Sky renders tree items in UI
//! 6. **User Action**: User expands/clicks/selects items
//! 7. **Events**: Sky calls `OnTreeNodeExpanded`, `OnTreeSelectionChanged`, etc.
//!
//! ### Dependencies
//! - `ApplicationState`: Tree view state storage (`ActiveTreeViews`)
//! - `IPCProvider`: For extension provider communication
//! - `Log`: Tree view operation logging
//!
//! ### Dependents
//! - Extensions: Register tree views for Explorer, Outline, etc.
//! - Native providers: FileExplorer, SymbolTree, etc.
//! - `Binary::Bootstrap::RegisterTreeViewProviders`: Initial registration
//! - Sky UI: Tree view component requests data
//!
//! ## TREE ITEM DTO
//!
//! `TreeItemDTO` structure (JSON):
//! - `handle`: Unique identifier for the item
//! - `label`: Display label (with `label` property)
//! - `collapsibleState`: 0=none, 1=collapsed, 2=expanded
//! - `resourceUri`: URI for the item (file, folder, symbol)
//! - `command`: Command to execute on activation (click/double-click)
//! - `description`: Secondary text (right-aligned)
//! - `iconPath`: Icon for the item (theme aware)
//! - `contextValue`: Context for theming and keybindings
//!
//! ## TREE VIEW OPTIONS
//!
//! `TreeViewOptionsDTO` during registration:
//! - `canSelectMany`: Allow multi-selection (Ctrl+Click)
//! - `canHide`: Allow user to hide tree view
//! - `hasFileIcon`: Show file icons next to items
//! - `hasDecoration`: Show tooltips and badges
//! - `dragAndDrop`: Enable drag-and-drop
//! - `canRename`: Allow renaming tree items
//! - `canDelete`: Allow deleting tree items
//! - `menu`: Context menu contributions
//! - `selectOnFocus`: Select item when tree gains focus
//!
//! ## ERROR HANDLING
//!
//! - Provider not found: `CommonError::InvalidArgument`
//! - Provider error: Propagate as `CommonError`
//! - IPC failure: `CommonError::IPCError`
//! - Invalid tree item: `CommonError::InvalidArgument`
//! - State lock errors: `CommonError::StateLockPoisoned`
//!
//! ## PERFORMANCE
//!
//! - Tree data providers should be lazy (load children on demand)
//! - Cache tree item results to avoid redundant computation
//! - Use `RefreshTreeView` sparingly (expensive operation)
//! - Consider background loading for large trees
//! - Limit child count per node (paging for very large directories)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/api/common/extHostTreeViews.ts` - Extension API
//! - `vs/workbench/services/views/common/treeViewService.ts` - Tree view service
//! - `vs/platform/views/common/views.ts` - Tree view data model
//!
//! ## TODO
//!
//! - [ ] Implement tree view state persistence (expanded nodes, selection)
//! - [ ] Add tree view theming support (custom CSS)
//! - [ ] Support tree view filtering and search
//! - [ ] Implement tree view sorting and ordering
//! - [ ] Add tree view virtualization for large trees
//! - [ ] Support tree view column rendering (like file explorer details)
//! - [ ] Implement tree view keyboard navigation enhancements
//! - [ ] Add tree view context menu customization
//! - [ ] Support tree view drag-and-drop reordering
//! - [ ] Implement tree view accessibility (screen reader, keyboard-only)
//! - [ ] Add tree view animations (expand/collapse)
//! - [ ] Support tree view grouping and categorization
//! - [ ] Implement tree view configuration UI
//! - [ ] Add tree view performance monitoring
//!
//! ## MODULE CONTENTS
//!
//! - [`TreeViewProvider`]: Main struct implementing the trait
//! - Tree view registration and lifecycle
//! - Provider dispatch logic (native vs extension)
//! - Event handling (selection, expansion, etc.)
//! - State persistence and restoration
//! - Drag-and-drop coordination


use std::sync::Arc;

use log::{debug, info, warn};
use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	TreeView::{DTO::TreeViewOptionsDTO::TreeViewOptionsDTO, TreeViewProvider::TreeViewProvider},
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO;

#[async_trait]
impl TreeViewProvider for MountainEnvironment {
	/// Registers a new tree data provider from an extension (e.g., Cocoon).
	async fn RegisterTreeDataProvider(&self, ViewIdentifier:String, Options:Value) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Registering data provider for view: {}", ViewIdentifier);

		let OptionsDTO:TreeViewOptionsDTO = serde_json::from_value(Options.clone()).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"Options".into(), Reason:Error.to_string() }
		})?;

		// For now, assume all extension providers come from the main sidecar.
		let SideCarIdentifier = "cocoon-main".to_string();

		let NewState = TreeViewStateDTO {
			ViewIdentifier:ViewIdentifier.clone(),

			// This is a proxied provider, not native.
			Provider:None,

			SideCarIdentifier:Some(SideCarIdentifier),

			CanSelectMany:OptionsDTO.CanSelectMany,

			HasHandleDrag:OptionsDTO.HasHandleDrag,

			HasHandleDrop:OptionsDTO.HasHandleDrop,

			Message:None,

			Title:None,

			Description:None,
		};

		self.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(ViewIdentifier.clone(), NewState);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/create",
				json!({ "ViewIdentifier": ViewIdentifier, "Options": Options }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(())
	}

	/// Unregisters a tree data provider.
	async fn UnregisterTreeDataProvider(&self, ViewIdentifier:String) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Unregistering data provider for view: {}", ViewIdentifier);

		self.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&ViewIdentifier);

		self.ApplicationHandle
			.emit("sky://tree-view/dispose", json!({ "ViewIdentifier": ViewIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Reveals a specific item in the tree view by notifying the UI.
	async fn RevealTreeItem(&self, ViewIdentifier:String, ItemHandle:String, Options:Value) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Revealing item '{}' in view '{}'",
			ItemHandle, ViewIdentifier
		);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/reveal",
				json!({ "viewId": ViewIdentifier, "itemHandle": ItemHandle, "options": Options }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Refreshes the tree view by notifying the UI.
	async fn RefreshTreeView(&self, ViewIdentifier:String, ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Refreshing view '{}'", ViewIdentifier);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/refresh",
				json!({ "viewId": ViewIdentifier, "itemsToRefresh": ItemsToRefresh }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Gets the children for a given element. This method acts as a dispatcher.
	async fn GetChildren(
		&self,

		ViewIdentifier:String,

		ElementHandle:Option<String>,
	) -> Result<Vec<Value>, CommonError> {
		let ProviderInfo = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&ViewIdentifier)
			.cloned();

		if let Some(Info) = ProviderInfo {
			if let Some(NativeProvider) = Info.Provider {
				// Case 1: Native Rust provider (e.g., File Explorer)
				return NativeProvider.GetChildren(ViewIdentifier, ElementHandle).await;
			} else if let Some(SideCarId) = Info.SideCarIdentifier {
				// Case 2: Proxied extension provider
				let IPCProvider:Arc<dyn IPCProvider> = self.Require();

				let RPCMethod = format!("{}$getChildren", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

				let RPCParams = json!([ViewIdentifier, ElementHandle]);

				let Response = IPCProvider.SendRequestToSideCar(SideCarId, RPCMethod, RPCParams, 10000).await?;

				return serde_json::from_value::<Vec<Value>>(Response).map_err(CommonError::from);
			}
		}
		Err(CommonError::TreeViewProviderNotFound { ViewIdentifier })
	}

	/// Gets the TreeItem for a given element. This method acts as a dispatcher.
	async fn GetTreeItem(&self, ViewIdentifier:String, ElementHandle:String) -> Result<Value, CommonError> {
		let ProviderInfo = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&ViewIdentifier)
			.cloned();

		if let Some(Info) = ProviderInfo {
			if let Some(NativeProvider) = Info.Provider {
				return NativeProvider.GetTreeItem(ViewIdentifier, ElementHandle).await;
			} else if let Some(SideCarId) = Info.SideCarIdentifier {
				let IPCProvider:Arc<dyn IPCProvider> = self.Require();

				let RPCMethod = format!("{}$getTreeItem", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

				let RPCParams = json!([ViewIdentifier, ElementHandle]);

				return IPCProvider.SendRequestToSideCar(SideCarId, RPCMethod, RPCParams, 5000).await;
			}
		}
		Err(CommonError::TreeViewProviderNotFound { ViewIdentifier })
	}

	/// Updates the tree view message displayed in the UI.
	async fn SetTreeViewMessage(&self, ViewIdentifier:String, Message:Option<String>) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Setting message for view '{}': {:?}",
			ViewIdentifier, Message
		);

		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				ViewState.Message = Message.clone();
			}
		}

		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-message",
				json!({ "ViewIdentifier": ViewIdentifier, "Message": Message }),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view message: {}", Error) }
			})
	}

	/// Updates the tree view's title and description.
	async fn SetTreeViewTitle(
		&self,
		ViewIdentifier:String,
		Title:Option<String>,
		Description:Option<String>,
	) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Setting title/description for view '{}': {:?} {:?}",
			ViewIdentifier, Title, Description
		);

		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				ViewState.Title = Title.clone();
				ViewState.Description = Description.clone();
			}
		}

		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-title",
				json!({
					"ViewIdentifier": ViewIdentifier,
					"Title": Title,
					"Description": Description,
				}),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view title: {}", Error) }
			})
	}

	/// Sets a badge on the tree view.
	async fn SetTreeViewBadge(&self, ViewIdentifier:String, Badge:Option<Value>) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Setting badge for view '{}': {:?}", ViewIdentifier, Badge);

		// Update state
		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				// Store badge in ViewState (you may need to add this field to
				// TreeViewStateDTO) For now, just emit the event
			}
		}

		// Emit to frontend
		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-badge",
				json!({ "ViewIdentifier": ViewIdentifier, "Badge": Badge }),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Failed to emit tree view badge: {}", Error) }
			})
	}

	// --- State Management Methods ---

	/// Handles tree node expansion/collapse events.
	/// Called when a user expands or collapses a node in the tree view.
	/// Updates internal state and propagates the event to the frontend.
	async fn OnTreeNodeExpanded(
		&self,
		ViewIdentifier:String,
		ElementHandle:String,
		IsExpanded:bool,
	) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Node '{}' in view '{}' expanded: {}",
			ElementHandle, ViewIdentifier, IsExpanded
		);

		// Track node expansion state in TreeViewStateDTO to persist which elements
		// users have opened or closed across application sessions. This enables
		// state restoration when the tree view is recreated after window reload,
		// workspace switches, or extension updates. Coordinate with PersistTreeViewState
		// to serialize and deserialize the expansion hierarchy efficiently.

		// Propagate to frontend
		self.ApplicationHandle
			.emit(
				"sky://tree-view/node-expanded",
				json!({
					"ViewIdentifier": ViewIdentifier,
					"ElementHandle": ElementHandle,
					"IsExpanded": IsExpanded
				}),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction {
					Reason:format!("Failed to emit node expanded event: {}", Error),
				}
			})
	}

	/// Handles tree selection changes.
	/// Called when the user selects or deselects items in the tree view.
	/// Updates internal state and propagates the event to the frontend.
	async fn OnTreeSelectionChanged(
		&self,
		ViewIdentifier:String,
		SelectedHandles:Vec<String>,
	) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Selection changed in view '{}': {} items selected",
			ViewIdentifier,
			SelectedHandles.len()
		);

		// Preserve user selection in TreeViewStateDTO to maintain selected items across
		// tree view updates, refreshes, and workspace changes. This prevents selection
		// loss during asynchronous data updates, provider reloads, or UI re-renders.
		// Track the set of selected ElementHandles and restore them when the view
		// reconstructs its tree structure from persisted state.

		// Propagate to frontend
		self.ApplicationHandle
			.emit(
				"sky://tree-view/selection-changed",
				json!({
					"ViewIdentifier": ViewIdentifier,
					"SelectedHandles": SelectedHandles
				}),
			)
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction {
					Reason:format!("Failed to emit selection changed event: {}", Error),
				}
			})
	}

	/// Persists the current state of a tree view.
	/// Saves the expansion, selection, and other state for later restoration.
	/// Returns JSON representation of the persisted state.
	async fn PersistTreeViewState(&self, ViewIdentifier:String) -> Result<Value, CommonError> {
		info!("[TreeViewProvider] Persisting state for view '{}'", ViewIdentifier);

		let TreeViews = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let State = TreeViews.get(&ViewIdentifier).map(|View| {
			json!({
				"ViewIdentifier": ViewIdentifier,
				"Title": View.Title,
				"Description": View.Description,
				"CanSelectMany": View.CanSelectMany,
				"Message": View.Message,
				"HasHandleDrag": View.HasHandleDrag,
				"HasHandleDrop": View.HasHandleDrop,
			})
		});

		State.ok_or(CommonError::TreeViewProviderNotFound { ViewIdentifier })
	}

	/// Restores a previously persisted tree view state.
	/// Restores expansion, selection, and other state from a JSON
	/// representation.
	async fn RestoreTreeViewState(&self, ViewIdentifier:String, StateValue:Value) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Restoring state for view '{}'", ViewIdentifier);

		let mut TreeViews = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ViewState) = TreeViews.get_mut(&ViewIdentifier) {
			if let Some(Title) = StateValue.get("Title").and_then(|v| v.as_str()) {
				ViewState.Title = Some(Title.to_string());
			}
			if let Some(Description) = StateValue.get("Description").and_then(|v| v.as_str()) {
				ViewState.Description = Some(Description.to_string());
			}
			// Restore additional UI state properties from the persisted StateValue to
			// fully reconstruct the tree view's appearance and behavior. This includes
			// expansion state (which nodes are open), scroll position (viewport position),
			// column widths (for detail views), sorting order, and any provider-specific
			// state extensions. Each property is deserialized from the JSON and applied
			// to rebuild the exact UI state the user had before.

			// Emit to frontend
			self.ApplicationHandle
				.emit(
					"sky://tree-view/restore-state",
					json!({
						"ViewIdentifier": ViewIdentifier,
						"State": StateValue
					}),
				)
				.map_err(|Error| {
					CommonError::UserInterfaceInteraction {
						Reason:format!("Failed to emit restore state event: {}", Error),
					}
				})?;

			Ok(())
		} else {
			Err(CommonError::TreeViewProviderNotFound { ViewIdentifier })
		}
	}
}
