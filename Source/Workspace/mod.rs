//! # Workspace Module
//!
//! Provides workspace-related services including workspace folder management,
//! `.code-workspace` file parsing, and multi-root workspace support.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Workspace Folder Management
//! - Track currently open workspace folders
//! - Support adding, removing, and reordering folders
//! - Validate folder paths and permissions
//! - Persist workspace state across sessions
//!
//! ### 2. Workspace File Parsing
//! - Parse `.code-workspace` JSON files
//! - Extract workspace configuration and settings
//! - Resolve workspace folder paths
//! - Validate workspace file structure
//!
//! ### 3. Multi-Root Workspace Support
//! - Manage workspaces with multiple root folders
//! - Handle folder-specific configuration overrides
//! - Support workspace-scoped settings
//! - Merge configuration from multiple sources
//!
//! ### 4. Workspace Configuration
//! - Provide merged workspace configuration
//! - Support workspace-specific settings
//! - Integrate with global configuration system
//! - Handle configuration precedence (workspace > folder > global)
//!
//! ### 5. Workspace Trust & Security
//! - Implement workspace trust validation
//! - Mark untrusted workspaces with restricted capabilities
//! - Provide prompt for user to trust/untrust workspaces
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Workspace module is the **workspace management layer**:
//!
//! ```text
//! UI (Explorer) ──► WorkspaceService ──► ApplicationState ──► Disk
//!                      │
//!                      └─► ConfigurationProvider
//! ```
//!
//! ### Position in Mountain
//! - `Workspace` module: Workspace lifecycle and configuration
//! - Implements `CommonLibrary::Workspace::WorkspaceProvider` trait
//! - Accessible via `Environment.Require<dyn WorkspaceProvider>()`
//!
//! ### Dependencies
//! - `ApplicationState`: Stores workspace state
//! - `ConfigurationProvider`: For workspace settings
//! - `FileSystemReader`: For reading `.code-workspace` files
//!
//! ### Dependents
//! - `InitializationData`: Uses workspace data for Cocoon initialization
//! - `ApplicationState`: Stores workspace folders and configuration path
//! - `Binary::Main`: Sets up workspace from CLI args
//!
//! ## WORKSPACE STATE
//!
//! Stored in `ApplicationState`:
//! - `WorkspaceFolders`: Vec<WorkspaceFolderStateDTO>
//! - `WorkspaceConfigurationPath`: Optional PathBuf to `.code-workspace` file
//! - `IsTrusted`: Trust status flag
//! - `WindowState`: Window geometry and placement
//!
//! ## MULTI-ROOT SUPPORT
//!
//! Mountain supports VS Code-style multi-root workspaces:
//! - Multiple folders can be opened simultaneously
//! - Each folder has a URI and display name
//! - Folders are indexed (0-based) for ordering
//! - Workspace configuration applies to all folders
//!
//! ## CONFIGURATION PRECEDENCE
//!
//! Configuration sources are merged in this order (later overrides earlier):
//! 1. Default settings (built-in)
//! 2. Global user settings (`settings.json`)
//! 3. Workspace settings (`.code-workspace` file)
//! 4. Folder settings (`folder/.vscode/settings.json`)
//! 5. Workspace-specific overrides per folder
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/platform/workspace/common/workspace.ts` - Workspace service
//! - `vs/platform/workspace/common/workspaceFolders.ts` - Workspace folder operations
//! - `vs/platform/configuration/common/configuration.ts` - Configuration merging
//!
//! ## ERROR HANDLING
//!
//! - Invalid workspace files: Return `CommonError::ConfigurationLoad`
//! - Permission errors: `CommonError::FileSystemIO`
//! - Unauthorized workspace: `CommonError::SecurityViolation`
//! - Missing folders: `CommonError::FileSystemNotFound`
//!
//! ## TODO
//!
//! - [ ] Implement workspace trust management with user prompts
//! - [ ] Add workspace template creation and management
//! - [ ] Support workspace sharing and collaboration features
//! - [ ] Add workspace statistics (folder count, file count, size)
//! - [ ] Implement workspace snapshots for quick restore
//! - [ ] Add workspace local history (file changes over time)
//! - [ ] Create workspace migration tools for version upgrades
//! - [ ] Support virtual workspaces (non-file URIs)
//! - [ ] Add workspace search across all folders
//! - [ ] Implement workspace-specific extensions
//!
//! ## MODULE CONTENTS
//!
//! - [`WorkspaceFileService`]: Parsing and serialization of `.code-workspace` files


pub mod WorkspaceFileService;
