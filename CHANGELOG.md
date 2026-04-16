# Changelog

All notable changes to Mountain (Rust Backend) are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/).

## [v2.1] — Q2 2026: Full Workbench Lift

### Added

- 14 language feature provider methods in `FeatureMethods.rs` (354 lines):
  renameEdits, documentSymbols, workspaceSymbols, signatureHelp, foldingRanges,
  selectionRanges, semanticTokensFull, inlayHints, typeHierarchySupertypes,
  typeHierarchySubtypes, callHierarchyIncoming, callHierarchyOutgoing,
  linkedEditingRanges, onTypeFormattingEdits
- 18 CocoonService stub handlers replaced with Tauri event implementations
  (`sky://` patterns for progress, webview, terminal, tree view, SCM, debug,
  tasks, auth, config, external)
- Extension registration notification handlers in MountainVinegRPCService:
  `window.showMessage`, `registerCommand`, provider registration fallback
- 5 ProviderType enum variants in Common: Task, Authentication, TreeView,
  SourceControl, DebugAdapter
- Startup extension activation trigger (`$activateByEvent("*")`) after Cocoon
  handshake

### Changed

- `dev_log!` macro replaced `log` crate across 170 files (1,724 insertions,
  1,699 deletions) with categorized tags: lifecycle, grpc, ipc, cocoon,
  extensions, config, model, storage, commands, output, terminal
- Vine gRPC binding changed from IPv6 `[::1]` to IPv4 `127.0.0.1` (Cocoon
  compatibility)
- WindServiceHandlers split into 24 atomic domain modules under
  `IPC/WindServiceHandler/`
- CocoonService split into 15 domain submodules under `RPC/CocoonService/`
- MutexGuard Send fix in FeatureMethods (scope narrowing across `.await`)

### Fixed

- TraceLog.rs syntax error, ConfigurationInitialize.rs broken imports
- AdvancedFeatures.rs + WindAdvancedSync.rs duplicate dev_log imports
- CompletionItem DTO missing `documentation` field
- Tauri CSP updated to include `vscode-file:` in `connect-src`
- node_modules path resolution in vscode-file scheme handler
- Cocoon connection retry intervals with race condition prevention

## [v2.0] — Q1 2026: Editor Launch Sprint

### January (374 commits): Foundation and Binary Module Architecture

#### Added

- `Source/Binary/` modular startup system:
  - `Tray/` — SwitchTrayIcon.rs, EnableTray.rs
  - `Shutdown/` — SchedulerShutdown.rs, RuntimeShutdown.rs
  - `Service/` — VineStart.rs (gRPC on 50051/50052), CocoonStart.rs,
    ConfigurationInitialize.rs
  - `Register/` — AdvancedFeaturesRegister.rs, IPCServerRegister.rs,
    StatusReporterRegister.rs, WindSyncRegister.rs
  - `Initialize/` — RuntimeBuild.rs, StateBuild.rs, CLI argument parser,
    dynamic port selection
- 20+ Tauri commands exposed: WorkbenchConfigurationQuery,
  DesktopConfigurationQuery, UpdateSubscriptionEndpoint, TrayIconSwitch,
  IPCStatusQuery, PerformanceStatistics, MessageReceptionEndpoint,
  GenericIPCMethodInvocation, DocumentSynchronization, CollaborationSession,
  StatusHistory
- TreeView foundation: stub interaction handlers, state persistence skeleton,
  Tauri event propagation to Sky
- Hover command module: `Source/Command/Hover/` with Interface.rs

### February (179 commits): Thread Safety and TreeView

#### Added

- TreeView implementation: concrete `OnTreeNodeExpanded`,
  `OnTreeSelectionChanged` with ApplicationState locks and event propagation
- TreeView badge support: Badge field in TreeViewStateDTO (max 2048 bytes)
- Hover feature: `Fn.rs` (164 lines) with hover request handling

#### Changed

- Thread safety refactoring: replaced `static mut` with `RwLock` in Scheme.rs
  (829 lines), ServiceRegistry.rs (820 lines), CertificateManager.rs (395
  lines), TlsCommands.rs (184 lines)
- Cleaned 150+ unused imports across 40+ provider files

### March (26 commits): Debug State and Protocol Alignment

#### Added

- `Source/ApplicationState/State/FeatureState/Debug/` — DebugState.rs (155
  lines) with DebugConfigurationProviderRegistration and
  DebugAdapterDescriptorFactoryRegistration
- TreeView badge: SetBadge() method with length validation

#### Fixed

- RPC error response: string → proper RpcError struct with JSON-RPC code -32601
- DTO deserialization: InputBoxOptionsDTO, OpenDialogOptionsDTO,
  SaveDialogOptionsDTO corrected from `>(obj.clone())` to
  `>(Value::Object(obj.clone()))`
- Vine protobuf field name alignment: RegisterTreeViewProviderRequest
  display_name → extension_id, GitExecRequest repository/cwd →
  repository_path/args

## [v1.3] — Q4 2025: Dependency Maintenance

### Added

- `gen/schemas/macOS-schema.json` — platform-specific capabilities for tray,
  menu bar, dock integration

### Changed

- actions/cache 4.3.0 → 5.0.1
- actions/checkout 5.0.0 → 6.0.1
- Regular `.github/Update.md` auto-increments

## [v1.2] — Q3 2025: Full Stack Integration

### Changed

- Error handling standardized across SourceControlManagement, DocumentProvider,
  TerminalProvider
- Sidecar naming conventions consolidated
- Build artifacts renamed with "22NodeVersion" label for Node.js versioning
- Windows installer naming standardized across debug/release profiles

## [v1.1] — Q2 2025: Architecture Buildout

**185 commits — the complete architecture implementation quarter.**

### Added

- `Source/Command/` — CommandRegistry, Keybinding, LanguageFeature,
  SourceControlManagement, TreeView, Bootstrap, Hover
- `Source/Environment/` — 14 provider implementations: ConfigurationProvider,
  DocumentProvider, FileSystemProvider, OutputProvider, SearchProvider,
  SecretProvider, StatusBarProvider, StorageProvider, TerminalProvider,
  TreeViewProvider, UserInterfaceProvider, WebviewProvider, WorkspaceProvider,
  Utility
- `Source/ProcessManagement/` — CocoonManagement, InitializationData
- `Source/RunTime/` — ApplicationRunTime executable lifecycle
- `Source/Track/` — DispatchLogic, EffectCreation (request routing and effect
  system)
- `Source/Update/` — UpdateService
- `Source/Workspace/` — WorkSpaceFileService
- `Source/FileSystem/` — FileExplorerViewProvider
- `Source/Vine/Server/` — MountainVinegRPCService, Initialize.rs
- `Source/Vine/Generated/vine_ipc.rs` — 1,714 lines auto-generated from
  Vine.proto
- Node.js sidecar bundled (Target/debug/node.exe, Target/release/node.exe)
- Build artifacts: .exe, .msi Windows installers

### Changed

- Error handling unified via CommonError + MapLockError pattern
- Module naming: handlers → Command, app_state → ApplicationState
- README rewritten with architecture overview (76 lines)
- `docs/Deep Dive.md` rewritten (182 lines)
- License transitioned to CC0 1.0 Universal

## [v1.0] — Q1 2025: Integration Phase

### Added

- `Source/ApplicationState/` reorganization:
  - `Internal/` — Persistence, Recovery, TextProcessing, PathResolution,
    ExtensionScanner, Serialization
  - `DTO/` — 12 DTO classes (DocumentStateDTO, TerminalStateDTO,
    TreeViewStateDTO, WindowStateDTO, etc.)
  - `State/` — organized feature state classes
- `.github/workflows/Auto.yml` (68 lines) — automated update/push CI
- `Knowledge.dot` + `Knowledge.svg` — module dependency graph (Graphviz)

### Changed

- Deprecated `Source/app_state/` → `Source/ApplicationState/`
- Cargo.toml: removed 45 redundant entries, added 17 new entries

## [v0.2] — Q4 2024: Architecture Solidification

### Added

- `gen/android/` — full Android Gradle project scaffold:
  - AndroidManifest.xml, MainActivity.kt
  - RustPlugin.kt, BuildTask.kt (Rust → Android cross-compilation)
  - Resources: drawable, mipmap (hdpi through xxxhdpi), layout, values
- `gen/schemas/mobile-schema.json` (11,162 lines)
- `gen/schemas/android-schema.json` (11,162 lines)
- Cargo.toml feature flags: AirIntegration, ExtensionHostCocoon, MistNative,
  Debug, grove, cocoon, terminals, debug-protocol, scm-support, Telemetry

## [v0.1] — Q3 2024: Rapid Development

### Changed

- Schema reduction: 8,467 → ~1,000 lines in ACL manifests (78% reduction)
- Removed bloated capabilities; kept only essential Tauri permissions
- Binary function refactoring: `Source/Fn/Binary.rs` 106 → ~20 lines
- Added `Source/Fn/Binary/Notes.md` (91 lines) architecture documentation

## [v0.0] — Q2 2024: Project Inception

### Added

- Project relocated from `Editor/editor/src-tauri/` to monorepo root
- `Source/Library.rs` — single Tauri entry point
- `tauri.conf.json` (94 lines) — bundle targets: Windows NSIS, macOS DMG, deb
- `build.rs` — Tauri build system integration
- `.github/workflows/Rust.yml` (78 lines), `GitHub.yml` (57 lines),
  `dependabot.yml`
- App icons: PNG, ICNS, ICO, Windows tile assets
- `capabilities/` — Tauri capabilities manifest
- `gen/schemas/` — desktop-schema.json, windows-schema.json

### Dependencies (First Release)

- Core: tauri, tokio, serde, serde_json, tonic (gRPC)
- Plugins: tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-localhost,
  tauri-plugin-log
- Serialization: prost (protobuf), bincode
- Crypto: sha2, md5, ring, rcgen, p256, x509-parser, rustls, pem
- Network: tokio-tungstenite, http, url
- Terminal: portable-pty
- Process: sysinfo, hostname
