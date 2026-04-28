# Changelog

All notable changes to Mountain (Rust Backend) are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/).

## [v2.1] - Q2 2026: Full Workbench Lift

### April 22, 2026

#### Added

- **Extension type filtering** (`Extensions.rs`): `getInstalled(type?)` IPC now
  respects the optional `ExtensionType` filter (0=System, 1=User) passed by
  VS Code's sidebar, correctly distinguishing VSIX-installed extensions from
  built-ins via the scanner's `isBuiltin` field. Previously the filter was
  silently dropped and every call returned the full list as `type: 0,
  isBuiltin: true`.
- **Post-install activation burst** (`Extension.rs`): After `$deltaExtensions`
  adds an extension to Cocoon's registry, `onStartupFinished` activation events
  are now fired. Without this burst, extensions with `onStartupFinished` (e.g.
  `Anthropic.claude-code`) registered but never activated — sidebar
  contributions and commands silently no-oped until the next full restart.
- **Sidebar scan handlers** (`mod.rs`): `extensions:scanSystemExtensions` ->
  `getInstalled(type=0)` and `extensions:scanUserExtensions` ->
  `getInstalled(type=1)` forwarding added so VSIX extensions appear under
  "Installed" with an Uninstall action.
- **`extensions:getManifest` IPC handler**: Reads `extension/package.json`
  from a `.vsix` archive without extracting to disk, enabling the "Install from
  VSIX..." preview dialog.
- **UriComponents helper module** (`UriComponents/`): Centralises every URI
  payload sent to the VS Code renderer with the required `$mid: 1` marshalling
  marker. Exposes `FromFilePath`, `FromUrl`, `StampMidUri`, and `Normalize`.
- **Extension manifest passthrough - Atom TH1** (`ExtensionDescriptionStateDTO`):
  14 new fields added for VS Code `package.json` metadata: `Categories`,
  `DisplayName`, `Description`, `Keywords`, `Repository`, `Bugs`, `Homepage`,
  `License`, `Icon`, `AiKey`, `ExtensionKind`, `Capabilities`,
  `ExtensionDependencies`, `ExtensionPack`.
- **IPC round-trip for `applyEdit`/`showTextDocument` - Atom T1**: Replaced
  fire-and-forget event emission with request/response pattern using
  `SendUserInterfaceRequest`.
- **Node version checking - Atom N1** (`NodeResolver`): `ResolveNodeBinary`
  now queries `node --version` and logs the resolved version; emits a warning
  when the major version falls below `Require` (default 20).
- **Compile-time PostHog config - Atom P2** (`PostHogPlugin`): API key, host,
  and enable flag are now read from `env!()` baked in `build.rs` via
  `PropagatePostHogSentinel()`.
- **IPC logging improvement - Atom I13**: Paired entry/exit log lines per
  invoke. `done: <cmd> ok=... t_ns=...` on exit enables latency diagnosis.
- **PostHog telemetry initialised at boot** (`Entry.rs`).
- **`BuildInitialUrl`** (`Entry.rs`): Reads
  `~/.land/workspaces/RecentlyOpened.json` and passes `?folder=<path>` to the
  initial webview URL, skipping the destructive reload-on-Welcome.
- **`NodeResolver` module - Atom N1**: Resolves Node.js binary in order:
  `Pick` env -> shipped runtime -> version managers (fnm, volta,
  asdf, nvm) -> Homebrew -> PATH fallback.
- **Extension path environment variables - Atom U1**: Five new env vars:
  `Skip`, `Ship`,
  `Lodge`, `Extend`,
  `Probe`.
- **Profile sentinel fix** (`build.rs`): Replaced runtime
  `std::env::var("Profile")` with compile-time `option_env!`.
- **`extensions:resetPinnedStateForAllUserExtensions` IPC handler** - Atom P1.
- **Atomic shutdown guard** (`Entry.rs`): Prevents graceful shutdown sequence
  from running twice on Tauri re-delivery of `ExitRequested { code: Some(0) }`.
- **Comprehensive Wind IPC handlers** (`WindServiceHandlers/`): Full surface
  for Commands, Configuration, Extensions, FileSystem, Model, NativeHost,
  Navigation history, Output channels, Search, Storage, Terminal/PTY, UI,
  and Utilities.
- **`CreateEffectForRequest` effect creators**: Domain modules added for
  Authentication, Clipboard, Commands, Configuration, Debug, Diagnostics,
  Documents, FileSystem, FileWatcher, Git, Keybinding, Languages, NativeHost,
  SCM, Search, Secrets, StatusBar, Storage, Task, Terminal, TreeView,
  UserInterface, Webview, WindowUI, Workspace.

#### Changed

- **`SkyEvent` typed constants**: Hardcoded `"sky://output/*"` string literals
  replaced with typed constants from `CommonLibrary::IPC::SkyEvent`.
- **Domain module split - Wind IPC**: 167 KB monolithic `WindServiceHandlers`
  broken into domain-specific modules.
- **Domain module split - Cocoon RPC**: 2,800-line monolithic `CocoonService`
  split into 16 domain modules.
- **Binary / product name simplified** (`Cargo.toml`, `tauri.conf.json`):
  Verbose dev profile identifier shortened back to `Mountain`.
- **posthog-rs 0.5 API**: `api_endpoint()` replaced with `host()`.
- **sha2 0.11 migration**: `LowerHex` impl replaced with `hex::encode()`.

#### Fixed

- **Extension location URI marshalling**: Extension locations now carry
  `$mid: 1` via the new UriComponents helper.
- **DTO serialization** (`ExtensionDescriptionStateDTO`): Removed
  `skip_serializing_if = "String::is_empty"` from `Name`, `Version`,
  `Publisher` — omitting the key crashed the renderer with
  `TypeError: undefined is not an object`.
- **Log file eager init** (`Entry.rs`, `DevLog.rs`): `InitEager()` called at
  binary startup. `BinarySignature()` fixed to correctly split PascalCase
  segments so logs land in the correct app-data directory.
- **Static asset path resolution** (`Utilities.rs`): Extended to handle both
  `/Static/Application/` and `Static/Application/` forms, fixing WASM loader
  ENOENT that broke TextMate syntax highlighting.
- **macOS window dragging**: Removed `maximized(true)`; replaced with
  `TitleBarStyle::Overlay` + `hidden_title(true)`.
- **ENOENT ignore-list expanded** (`DevLog.rs`): Added `chatLanguageModels.json`,
  window log files, `.copilot/agents`, `.vscode/tasks.json`, `/User/mcp.json`,
  `vscode-chat-images`, `/output_<TIMESTAMP>` patterns.

---

### April 21, 2026

#### Added

- **Local VSIX installer - Atoms K2/K3** (`VsixInstaller.rs`): Unpacks `.vsix`
  archives into `~/.land/extensions`. Two-pass extraction reads the manifest
  before writing files; includes zip-slip protection. Notifies Cocoon via
  `$deltaExtensions` for hot-activation without reload.
- **Kernel/minimal profile** (`ScanPathConfigure`): `Skip`
  env var skips built-in extension scan paths.
- **`Spawn=false` flag**: Skips extension host spawn entirely.
- **Environment forwarding to Cocoon**: `NODE_ENV`, `Trace`,
  `TAURI_ENV_DEBUG` now propagate to the Cocoon subprocess.
- **`.env.Land` bootstrap - Atom I5** (`AppLifecycle`): Loads environment from
  `.env.Land` probing cwd and repo-layout ancestors (up to 6 levels).
- **Zombie-Cocoon prevention - Atom I6**: Pre-boot `SweepStaleCocoon()` TCP-
  probes port 50052 and SIGTERMs/SIGKILLs a stale process. Post-shutdown
  `HardKillCocoon()` prevents `EADDRINUSE` cascades.
- **Workspace activation fix - Atom I1**: Before reload, `ApplicationState.Workspace`
  is mutated and `$deltaWorkspaceFolders` broadcast to Cocoon.
- **`security.workspace.trust.enabled=false` default** (`AppLifecycle`):
  Written to `User/settings.json` on first boot only if the key is absent.
- **BATCH-16 latency instrumentation** (`CreateEffectForRequest`).

#### Changed

- **Cocoon health monitor**: No longer floods logs after a crash.
- **AppLifecycle fallback timers**: Extended from 2s -> 8s and 5s -> 15s.
- **TerminalProvider**: Captures and logs PID + exit status code.
- **DevLog timestamp**: Replaced manual UTC with `chrono::Local`.

#### Fixed

- **MIME type override** (`LocalhostPlugin`): Explicit MIME overrides for
  JS/CSS/JSON/HTML/SVG assets.
- **Extension scan paths** (`ScanPathConfigure`): Removed `debug_only` gate.

---

### April 20, 2026

#### Added

- **Tier-gating system** (`Cargo.toml` feature flags, `build.rs`,
  `LandFixTier.rs`): Compile-time tier selection with runtime banner.
- **Native file watcher - TierFileWatcherLayer4** (`FileWatcherProvider`):
  Backed by the `notify` crate. Includes debouncing, glob->regex pattern
  filtering, and IPC forwarding to Cocoon as `$fileWatcher:event`.
- **Workspace folder runtime management**: CLI parsing via
  `ParseWorkspaceFolders` (`--folder` flags, positional dirs,
  `Open` env).
- **DevLog file sink**: Logs to
  `~/Library/Application Support/<bundle>/logs/<timestamp>/Mountain.dev.log`.
  Enabled via `Record=1`.
- **Completed previously-stubbed IPC handlers**: `Terminal.Resize`,
  `Clipboard.Read`/`Write`, `NativeHost.OpenExternal`, Webview operations,
  Tree operations, `Task.Fetch`/`Execute`, `Authentication.GetSession`,
  `Languages.GetAll`, `Debug.Stop`, `showOpenDialog`.

---

### April 18, 2026

#### Changed

- **`dev_log!` macro formatting**: All invocations reformatted from single-line
  to multi-line across ~150 call sites. No functional changes.
- **Module encapsulation**: Removed wildcard `pub use` re-exports from
  `ApplicationState` internal modules and the Air module.

#### Fixed

- **`.js.map` 204 handling** in `vscode-file` scheme handler.
- **Extension scanner logging**: First-run non-existent directory is now
  `debug`-level instead of `warn`.

---

### April 17, 2026

#### Added

- **`vscode-file://` absolute OS path handling**: Detects macOS/Linux absolute
  path roots inside `vscode-file://vscode-app/` URI and serves files directly
  from disk. Fixes extension-contributed icon themes, grammars, and woff fonts.
- **NLS placeholder resolution in extension scanner**: Loads `package.nls.json`
  and recursively replaces `%key%` placeholders before parsing.
- **FileSystem effects** (`CreateEffectForRequest`): `FileSystem.Stat`,
  `CreateDirectory`, `Delete`, `Rename`, `Copy`.
- **Extension scanning instrumentation**: Comprehensive logging across scanner,
  environment, and handler layers.

#### Changed

- **PascalCase naming migration - full RPC subsystem**: All remaining
  snake_case `.rs` files renamed. Deprecated service modules removed.
- **PascalCase migration - LanguageFeature and Environment modules**: 10 files
  renamed (`validation.rs` -> `Validation.rs`, etc.).
- **Sky target path resolution corrected**: `../../../Element/Sky/Target` ->
  `../../../Sky/Target` in `ScanPathConfigure.rs`, `AppLifecycle.rs`,
  `InitializationData.rs`.

#### Fixed

- **Resource not found 404s** downgraded from `warn` to `info`.
- **Empty `ticks` counter** removed from Cocoon health monitor.

---

### April 16, 2026

#### Added

- **Extension registration notification handlers** (`MountainVinegRPCService`):
  `window.showMessage` forwards to Sky via `sky://notification/show`;
  `registerCommand` stores proxied commands in `CommandRegistry`; provider
  registration fallback for typed RPC path.
- **Full language feature provider delegation** (`CocoonService`): 18 stub
  handlers replaced with actual delegations to `LanguageFeatureProviderRegistry`
  covering document highlights, symbols, workspace symbols, rename edits,
  formatting (document/range/on-type), signature help, code lenses, folding
  ranges, selection ranges, semantic tokens, inlay hints, type hierarchy,
  call hierarchy, linked editing ranges.
- **Remaining 14 `FeatureMethods` implementations**: All TODO stubs replaced.
  Each delegates to `FeatureMethods` -> `ProviderLookup::get_matching_provider`
  -> `invoke_provider`.

#### Changed

- **`dev_log!` macro replaces `log` crate** across entire Mountain codebase.
- **Domain module split - Wind IPC**: 167 KB `WindServiceHandlers` broken into
  24 focused domain modules.
- **Domain module split - Cocoon RPC**: 2,800-line `CocoonService` split into
  15 domain modules.
- **Vine gRPC binding** changed from IPv6 `[::1]` to IPv4 `127.0.0.1`.
- **Startup extension activation trigger** (`$activateByEvent("*")`) added
  after Cocoon handshake.

#### Fixed

- **`MutexGuard` Send fix** (`FeatureMethods.rs`): Lock scope narrowed across
  `.await` point.
- **Provider registration** fixed to use `self.RunTime.Environment`.
- **Command registration locking** corrected in `CommandRegistry`.

---

### April 13, 2026

#### Added

- **Extension host message forwarding to Cocoon** (`cb4538625`):
  `cocoon:extensionHostMessage` IPC handler extracts payload from Wind IPC
  args and forwards it to Cocoon's gRPC server via
  `Vine::Client::SendNotification` RPC as a generic notification. Completes
  the Wind -> Mountain -> Cocoon message relay path for the extension host
  protocol. Fire-and-forget to maintain async protocol behaviour.

---

### April 11, 2026

#### Added

- **PostHog integration - `PostHogPlugin.rs`** (`f5b654911`): New plugin for
  debug-build analytics capturing lifecycle events, IPC commands, and errors
  to EU Cloud.
- **OTLP proxy** (`LocalhostPlugin`): Forwards `/v1/traces` requests to local
  OTLP collector (`127.0.0.1:4318`) via raw TCP, enabling `OTELBridge.ts` to
  send telemetry without CORS issues.
- **38 granular `dev_log!` tags** (`d92cd0a4`): `DevLog` system expanded from
  8 to 38 tags covering all service areas: `terminal`, `extensions`, `search`,
  `themes`, `window`, `nativehost`, `clipboard`, `commands`, `model`, `output`,
  `notification`, `progress`, `quickinput`, `workingcopy`, `workspaces`,
  `keybinding`, `label`, `history`, `decorations`, `textfile`, `update`,
  `encryption`, `menubar`, `url`, `grpc`, `cocoon`, `bootstrap`, `preload`.
  Enables `Trace=terminal,exthost ./Mountain` filtering.
- **Short mode for dev logging** (`2348f18a`): `Trace=short` compresses
  Rust module targets (`D::Binary::Main::Entry` -> `Entry`), aliases verbose
  app-data path to `$APP`, and deduplicates consecutive identical messages with
  `(xN)` suffix.
- **`cocoon:extensionHostMessage` IPC handler stub** (`4a2c91af`): Wind ->
  Mountain -> Cocoon communication pathway established.
- **Cocoon bootstrap improvements** (`d92cd0a4`): `CocoonManagement` now
  resolves bootstrap script from Tauri resources (prod) or relative to
  executable (dev), preserves `PATH`/`HOME` env vars, adds orphan detection
  for parent process.

#### Changed

- **Binary / product name simplified** (`f5b654911`): Verbose development
  profile identifier shortened to `Mountain`. Tauri identifier updated to
  `land.editor.binary`.
- **Vine protocol PascalCase** (`f5b654911`): All RPC methods renamed from
  snake_case to PascalCase (e.g. `initial_handshake` -> `InitialHandshake`).
  `vine.rs` regenerated.
- **Session timestamp** fixed to be generated once per process lifetime.

#### Fixed

- **Graceful Cocoon degradation** (`b0dcc091`): `CocoonStart` now degrades
  gracefully when Cocoon sidecar is unavailable - logs a warning instead of an
  error and returns `Ok(())`.
- **OTLP/PostHog production gate**: `EmitOTLPSpan` returns early in release
  builds. `PostHogCapture` and `OTLPFlush` return early when
  `NODE_ENV=production`.
- **VS Code extension dependency path** (`80e7dcf1`): Corrected from
  `../../../Dependency` to `../../../../Dependency` in `ScanPathConfigure`.

---

### April 10, 2026

#### Added

- **`/Static/Application` path mapping** (`75ae6687`): `normalize_uri_path`
  now resolves `/Static/Application/...` paths to the real Sky Target
  directory. Pre-creates system extensions directory to avoid `ENOENT` on scan.
- **Session log window directory** created at startup so VS Code can write
  output files without `stat` errors.

#### Fixed

- **`nativeHost:getEnvironmentPaths`** returns `logs` under `userDataDir`
  with session timestamps (VS Code's expected structure) instead of Tauri's
  separate `app_log_dir()`.

---

### April 9, 2026

#### Added

- **`get_static_application_root()`** helper for Sky Target path access.

#### Changed

- **Cocoon connection retry intervals** increased with delay for race condition
  prevention during startup sequencing.

---

### April 6, 2026

#### Added

- **Navigation history state** (`ee41bafb`): New `NavigationHistoryState`
  module tracks editor navigation stack with cursor-based back/forward history.
  IPC handlers: `goBack`, `goForward`, `canGoBack`, `canGoForward`, `push`,
  `clear`, `getStack`.
- **Label resolution IPC handlers** (`ee41bafb`): `getUri`, `getWorkspace`,
  `getBase` - human-readable display labels for URIs.
- **Text model registry IPC handlers** (`ee41bafb`): `open`, `close`, `get`,
  `getAll`, `updateContent` - document state management.
- **Comprehensive IPC handlers and gRPC routers** (`028ed5fb`): Full VS Code
  extension API surface across Wind IPC (Storage, Notifications, Progress,
  QuickInput, Workspaces, Themes, Search) and Cocoon gRPC (Commands, Window,
  Workspace, Secrets, FS aliases, 20+ LSP provider registration types).
- **Generic request router** (`030b3c73`): `process_mountain_request` routes
  `fs.*` and `commands.execute` methods - handles `fs.readFile`, `fs.writeFile`,
  `fs.stat`, `fs.listDir`, `fs.readdir`, `fs.createDir`, `fs.delete`,
  `fs.rename`, `fs.move`, `commands.execute`.
- **Terminal I/O, secret storage, and UI dialogs** (`58b532fc`): `terminal_input`
  sends bytes to PTY stdin; `close_terminal` disposes PTY; `get_secret`/
  `store_secret`/`delete_secret` delegate to OS keychain; `show_quick_pick`
  and `show_input_box` present UI via `UserInterfaceProvider`.
- **Cocoon gRPC file system and command registry** (`75fdf60b`): `read_file`,
  `write_file`, `stat`, `readdir` delegate to `tokio::fs`. `delete_file`,
  `rename_file`, `copy_file`, `create_directory` with atomic parent dir
  creation. `find_files` uses `globset` crate. `git_exec` spawns native git.
  All 15+ language feature provider registration methods now call
  `RegisterProvider`.
- **Output channel events and configuration retrieval** (`65a75298`): Output
  operations emit Tauri events to Sky (`sky://output/*`). `get_configuration`
  delegates to `ConfigurationProvider` with dot-notation key.
- **Terminal, output channel, and text file Wind IPC handlers** (`6ed9f102`).

#### Changed

- **Workspace folder API sync -> async** (`8b7e172c`): `GetFolders().await`
  replaced with `GetWorkspaceFolders()`. `addFolder`/`removeFolder` use direct
  manipulation via `GetWorkspaceFolders`/`SetWorkspaceFolders`.
- **`opentelemetry` `trace` feature** enabled in `Cargo.toml`.

#### Fixed

- **Provider selector format** (`e7f2b53d`): `RegisterProvider` updated to
  canonical selector format `[{ "language": ... }]`. `SideCarIdentifier`
  always routes to `"cocoon-main"`.
- **Language inference fallback** (`e7f2b53d`): `ProviderLookup` derives
  language from file extension (`.rs`->rust, `.ts`->typescript, etc.) when
  document is not in open state.
- **Dialog options DTO structure** (`8e5d660e`): Options wrapped in `Base`
  field with `DialogOptionsDTO` containing `Title`. Path strings use
  `.display()` for proper conversion.
- **Storage `DeleteItem`** (`241b1322`): Replaced non-existent method with
  `UpdateStorageValue(true, Key, None)`.
- **`QuickInput` field names** (`241b1322`): `Placeholder` -> `PlaceHolder`.
- **Workspace state access** (`241b1322`): All handlers updated to use
  `ApplicationState.Workspace`.

---

### April 5, 2026

#### Added

- **Vine gRPC protocol expansion** (`3c11448`): Comprehensive gRPC service
  definitions and stubs for full VS Code extension host capability - Language
  Features (document/workspace symbols, rename, formatting, signature help,
  code lenses, folding/selection ranges, semantic tokens, inlay hints, type/
  call hierarchy, linked editing ranges), Window Operations (quick pick, input
  box, progress, webview messaging, external URI), File System (delete, rename,
  copy, create directory), Output Channels, Tasks, Authentication, Extensions,
  Terminal resize, Configuration.

#### Changed

- **Binary entry point extracted** (`79b20a44`): `main()` moved from
  `Library.rs` into new `Source/Binary.rs`. Separates library API from binary
  configuration.

#### Fixed

- **Import reorganization** (`7af83147`): `CocoonService` protobuf imports
  grouped by category with clear section comments.
- **Doc comments** fixed across Command, Environment, IPC, and Workspace
  modules - bracket-style references replaced with backtick code spans.

---

### April 4, 2026

#### Changed

- **README overhauled** (`1ec8ceda`): New header "The Bedrock of Land". Key
  Features section expanded. Architecture table transformed into Core
  Architecture Principles. Deep Dive & Component Breakdown section added.
  Mermaid diagram improved. Development Setup instructions added.
  Documentation URLs updated to `Editor.Land` domain.
- **README technology badges** (`e38fa3a1`): Rust, Tauri, and gRPC badge icons
  now self-hosted at `editor.land` instead of `cdn.simpleicons.org`.
- **`Library.rs` rustdoc** (`065ee6baf`): Replaced technical spec copy with
  developer-facing value propositions. "See Also" section added.

---

### Earlier v2.1 (pre-April 4)

- 14 language feature provider methods in `FeatureMethods.rs` (354 lines):
  `renameEdits`, `documentSymbols`, `workspaceSymbols`, `signatureHelp`,
  `foldingRanges`, `selectionRanges`, `semanticTokensFull`, `inlayHints`,
  `typeHierarchySupertypes`, `typeHierarchySubtypes`, `callHierarchyIncoming`,
  `callHierarchyOutgoing`, `linkedEditingRanges`, `onTypeFormattingEdits`
- 18 CocoonService stub handlers replaced with Tauri event implementations
- Extension registration notification handlers in `MountainVinegRPCService`
- 5 `ProviderType` enum variants in Common: Task, Authentication, TreeView,
  SourceControl, DebugAdapter
- `dev_log!` macro replaced `log` crate across 170 files with categorised tags
- `MutexGuard` Send fix in `FeatureMethods` (scope narrowing across `.await`)
- `CompletionItem` DTO missing `documentation` field added
- Cocoon connection retry intervals with race condition prevention

---

## [v2.0] - Q1 2026: Editor Launch Sprint

### January (374 commits): Foundation and Binary Module Architecture

#### Added

- `Source/Binary/` modular startup system:
  - `Tray/` - SwitchTrayIcon.rs, EnableTray.rs
  - `Shutdown/` - SchedulerShutdown.rs, RuntimeShutdown.rs
  - `Service/` - VineStart.rs (gRPC on 50051/50052), CocoonStart.rs,
    ConfigurationInitialize.rs
  - `Register/` - AdvancedFeaturesRegister.rs, IPCServerRegister.rs,
    StatusReporterRegister.rs, WindSyncRegister.rs
  - `Initialize/` - RuntimeBuild.rs, StateBuild.rs, CLI argument parser,
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

- `Source/ApplicationState/State/FeatureState/Debug/` - DebugState.rs (155
  lines) with DebugConfigurationProviderRegistration and
  DebugAdapterDescriptorFactoryRegistration
- TreeView badge: SetBadge() method with length validation

#### Fixed

- RPC error response: string -> proper RpcError struct with JSON-RPC code -32601
- DTO deserialization: InputBoxOptionsDTO, OpenDialogOptionsDTO,
  SaveDialogOptionsDTO corrected
- Vine protobuf field name alignment: RegisterTreeViewProviderRequest
  display_name -> extension_id, GitExecRequest repository/cwd ->
  repository_path/args

## [v1.3] - Q4 2025: Dependency Maintenance

### Added

- `gen/schemas/macOS-schema.json` - platform-specific capabilities for tray,
  menu bar, dock integration

### Changed

- actions/cache 4.3.0 -> 5.0.1
- actions/checkout 5.0.0 -> 6.0.1
- Regular `.github/Update.md` auto-increments

## [v1.2] - Q3 2025: Full Stack Integration

### Changed

- Error handling standardized across SourceControlManagement, DocumentProvider,
  TerminalProvider
- Sidecar naming conventions consolidated
- Build artifacts renamed with "22NodeVersion" label for Node.js versioning
- Windows installer naming standardized across debug/release profiles

## [v1.1] - Q2 2025: Architecture Buildout

**185 commits - the complete architecture implementation quarter.**

### Added

- `Source/Command/` - CommandRegistry, Keybinding, LanguageFeature,
  SourceControlManagement, TreeView, Bootstrap, Hover
- `Source/Environment/` - 14 provider implementations: ConfigurationProvider,
  DocumentProvider, FileSystemProvider, OutputProvider, SearchProvider,
  SecretProvider, StatusBarProvider, StorageProvider, TerminalProvider,
  TreeViewProvider, UserInterfaceProvider, WebviewProvider, WorkspaceProvider,
  Utility
- `Source/ProcessManagement/` - CocoonManagement, InitializationData
- `Source/RunTime/` - ApplicationRunTime executable lifecycle
- `Source/Track/` - DispatchLogic, EffectCreation (request routing and effect
  system)
- `Source/Update/` - UpdateService
- `Source/Workspace/` - WorkSpaceFileService
- `Source/FileSystem/` - FileExplorerViewProvider
- `Source/Vine/Server/` - MountainVinegRPCService, Initialize.rs
- `Source/Vine/Generated/vine_ipc.rs` - 1,714 lines auto-generated from
  Vine.proto
- Node.js sidecar bundled (Target/debug/node.exe, Target/release/node.exe)
- Build artifacts: .exe, .msi Windows installers

### Changed

- Error handling unified via CommonError + MapLockError pattern
- Module naming: handlers -> Command, app_state -> ApplicationState
- README rewritten with architecture overview (76 lines)
- `docs/Deep Dive.md` rewritten (182 lines)
- License transitioned to CC0 1.0 Universal

## [v1.0] - Q1 2025: Integration Phase

### Added

- `Source/ApplicationState/` reorganization:
  - `Internal/` - Persistence, Recovery, TextProcessing, PathResolution,
    ExtensionScanner, Serialization
  - `DTO/` - 12 DTO classes (DocumentStateDTO, TerminalStateDTO,
    TreeViewStateDTO, WindowStateDTO, etc.)
  - `State/` - organized feature state classes
- `.github/workflows/Auto.yml` (68 lines) - automated update/push CI
- `Knowledge.dot` + `Knowledge.svg` - module dependency graph (Graphviz)

### Changed

- Deprecated `Source/app_state/` -> `Source/ApplicationState/`
- Cargo.toml: removed 45 redundant entries, added 17 new entries

## [v0.2] - Q4 2024: Architecture Solidification

### Added

- `gen/android/` - full Android Gradle project scaffold:
  - AndroidManifest.xml, MainActivity.kt
  - RustPlugin.kt, BuildTask.kt (Rust -> Android cross-compilation)
  - Resources: drawable, mipmap (hdpi through xxxhdpi), layout, values
- `gen/schemas/mobile-schema.json` (11,162 lines)
- `gen/schemas/android-schema.json` (11,162 lines)
- Cargo.toml feature flags: AirIntegration, ExtensionHostCocoon, MistNative,
  Debug, grove, cocoon, terminals, debug-protocol, scm-support, Telemetry

## [v0.1] - Q3 2024: Rapid Development

### Changed

- Schema reduction: 8,467 -> ~1,000 lines in ACL manifests (78% reduction)
- Removed bloated capabilities; kept only essential Tauri permissions
- Binary function refactoring: `Source/Fn/Binary.rs` 106 -> ~20 lines
- Added `Source/Fn/Binary/Notes.md` (91 lines) architecture documentation

## [v0.0] - Q2 2024: Project Inception

### Added

- Project relocated from `Editor/editor/src-tauri/` to monorepo root
- `Source/Library.rs` - single Tauri entry point
- `tauri.conf.json` (94 lines) - bundle targets: Windows NSIS, macOS DMG, deb
- `build.rs` - Tauri build system integration
- `.github/workflows/Rust.yml` (78 lines), `GitHub.yml` (57 lines),
  `dependabot.yml`
- App icons: PNG, ICNS, ICO, Windows tile assets
- `capabilities/` - Tauri capabilities manifest
- `gen/schemas/` - desktop-schema.json, windows-schema.json

### Dependencies (First Release)

- Core: tauri, tokio, serde, serde_json, tonic (gRPC)
- Plugins: tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-localhost,
  tauri-plugin-log
- Serialization: prost (protobuf), bincode
- Crypto: sha2, md5, ring, rcgen, p256, x509-parser, rustls, pem
- Network: tokio-tungstenite, http, url
- Terminal: portable-pty
- Process: sysinfo, hostname
