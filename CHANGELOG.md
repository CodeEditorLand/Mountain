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
- **Sidebar scan handlers** (`mod.rs`): `extensions:scanSystemExtensions` →
  `getInstalled(type=0)` and `extensions:scanUserExtensions` →
  `getInstalled(type=1)` forwarding added so VSIX extensions appear under
  "Installed" with an Uninstall action.
- **`extensions:getManifest` IPC handler**: Reads `extension/package.json`
  from a `.vsix` archive without extracting to disk, enabling the "Install from
  VSIX…" preview dialog.
- **UriComponents helper module** (`UriComponents/`): Centralises every URI
  payload sent to the VS Code renderer with the required `$mid: 1` marshalling
  marker. Exposes `FromFilePath`, `FromUrl`, `StampMidUri`, and `Normalize`.
  Now used consistently in `handle_extensions_install` (Extension.rs) and both
  `handle_file_realpath` call sites (WindServiceHandler and
  WindServiceHandlers/FileSystem.rs).
- **Extension manifest passthrough — Atom TH1** (`ExtensionDescriptionStateDTO`):
  14 new fields added for VS Code `package.json` metadata: `Categories`,
  `DisplayName`, `Description`, `Keywords`, `Repository`, `Bugs`, `Homepage`,
  `License`, `Icon`, `AiKey`, `ExtensionKind`, `Capabilities`,
  `ExtensionDependencies`, `ExtensionPack`. Wind's Extensions sidebar filter
  `@builtin category:themes` now works.
- **IPC round-trip for `applyEdit`/`showTextDocument` — Atom T1**: Replaced
  fire-and-forget event emission with request/response pattern using
  `SendUserInterfaceRequest`. Extensions awaiting `workspace.applyEdit()` now
  block until Sky actually applies the edit, fixing races with listeners
  expecting post-apply state.
- **Node version checking — Atom N1** (`NodeResolver`): `ResolveNodeBinary`
  now queries `node --version` and logs the resolved version; emits a warning
  when the major version falls below `LAND_NODE_MIN_MAJOR` (default 20).
- **Compile-time PostHog config — Atom P2** (`PostHogPlugin`): API key, host,
  and enable flag are now read from `env!()` baked in `build.rs` via
  `PropagatePostHogSentinel()`. Supports `LAND_POSTHOG_DISTINCT_ID` for CI
  correlation. Gated behind `debug_assertions` for a cheaper no-op path in
  release.
- **IPC logging improvement — Atom I13**: Paired entry/exit log lines per
  invoke. `done: <cmd> ok=... t_ns=...` on exit enables latency diagnosis via
  single `grep` without Jaeger.
- **PostHog telemetry initialised at boot** (`Entry.rs`): `PostHogPlugin` now
  runs before CLI parsing and state init to capture crash context in telemetry.
- **`BuildInitialUrl`** (`Entry.rs`): Reads
  `~/.land/workspaces/RecentlyOpened.json` and passes `?folder=<path>` to the
  initial webview URL, skipping the destructive reload-on-Welcome that wiped
  workbench state, caused the purple splash flash, empty `@builtin` sidebar,
  and broken keybindings.
- **`NodeResolver` module — Atom N1**: Resolves Node.js binary for Cocoon spawn
  in order: `LAND_NODE_BINARY` env override → shipped runtime (Tauri bundle or
  dev-tree sibling) → version managers (fnm, volta, asdf, nvm) → Homebrew →
  PATH fallback. Logs resolution source for forensics.
- **Extension path environment variables — Atom U1**: Five new env vars for
  controlling extension scan paths: `LAND_DISABLE_BUILTIN_EXTENSIONS`,
  `LAND_BUILTIN_EXTENSIONS_DIR`, `LAND_USER_EXTENSIONS_DIR`,
  `LAND_EXTRA_EXTENSIONS_DIRS` (multi-path, platform separator),
  `LAND_DEV_EXTENSIONS_DIR`.
- **Profile sentinel fix** (`build.rs`): Replaced runtime
  `std::env::var("LAND_PROFILE")` with compile-time `option_env!` so the
  profile resolves correctly after binary launch (previously showed "unknown"
  when run outside `Build.sh`).
- **`extensions:resetPinnedStateForAllUserExtensions` IPC handler** — Atom P1.
- **Atomic shutdown guard** (`Entry.rs`): Prevents the graceful shutdown
  sequence from running twice. The graceful path ends with `app_handle.exit(0)`
  which Tauri re-delivers as a second `ExitRequested { code: Some(0) }`;
  re-entry now skips `prevent_exit` and the shutdown task.
- **Comprehensive Wind IPC handlers** (`WindServiceHandlers/`): Full surface
  for Commands, Configuration, Extensions, FileSystem (legacy + native), Model,
  NativeHost, Navigation history, Output channels, Search, Storage,
  Terminal/PTY, UI features, and Utilities (path resolution, percent-decoding,
  metadata conversion).
- **`CreateEffectForRequest` effect creators**: Domain modules added for
  Authentication, Clipboard, Commands, Configuration, Debug, Diagnostics,
  Documents, FileSystem, FileWatcher, Git, Keybinding, Languages, NativeHost,
  SCM, Search, Secrets, StatusBar, Storage, Task, Terminal, TreeView,
  UserInterface, Webview, WindowUI, Workspace.

#### Changed

- **`SkyEvent` typed constants** (`ChannelContent.rs`, `ChannelLifecycle.rs`):
  Hardcoded `"sky://output/*"` string literals replaced with typed constants
  from `CommonLibrary::IPC::SkyEvent` (`OutputAppend`, `OutputReplace`,
  `OutputClear`, `OutputCreate`, `OutputDispose`).
- **Domain module split — Wind IPC** (`WindServiceHandlers.rs`): 167 KB
  monolithic file broken into domain-specific modules: Commands, Configuration,
  Extensions, FileSystem, Model, NativeHost, Navigation, Output, Search,
  Storage, Terminal, UI, Utilities.
- **Domain module split — Cocoon RPC** (`CocoonService/mod.rs`): 2 800-line
  monolithic impl split into: Auth, Command, Debug, Extension, FileSystem,
  Initialization, Output, Provider, Save, SCM, Secret, Task, Terminal,
  TreeView, Window, Workspace.
- **Binary / product name simplified** (`Cargo.toml`, `tauri.conf.json`):
  Verbose development profile identifier shortened back to `Mountain`. Tauri
  identifier updated to `land.editor.binary`. Node sidecar reference removed
  (`Binary/node` is now handled differently). Backup files
  (`Cargo.toml.Backup`, `tauri.conf.json.Backup`) deleted.
- **posthog-rs 0.5 API**: `api_endpoint()` replaced with `host()` in
  `PostHogPlugin.rs`.
- **sha2 0.11 migration** (`ConfigurationBridge.rs`): Removed `LowerHex` impl
  replaced with `hex::encode()` for session ID generation.
- **`#[tauri::command]` attribute removed** from internal IPC dispatcher in
  `mod.rs`; docstring added clarifying it is not directly reachable from the
  frontend.

#### Fixed

- **Extension location URI marshalling**: Extension locations sent to the
  renderer now carry `$mid: 1` via the new UriComponents helper; previously the
  sidebar silently filtered entire batches where `.fsPath` and `.with` were
  undefined.
- **DTO serialization** (`ExtensionDescriptionStateDTO`): Removed
  `skip_serializing_if = "String::is_empty"` from `Name`, `Version`,
  `Publisher`. VS Code's trusted-publishers migration calls
  `manifest.publisher.toLowerCase()` at boot — omitting the key crashed the
  renderer with `TypeError: undefined is not an object`.
- **IPC handler hardening** (`Extensions.rs`): Empty/missing `publisher`/`name`
  now fall back to "unknown"; explicit `publisher`, `name`, `version` injected
  into manifest before renderer send; null manifest values substituted with
  empty skeleton.
- **Log file eager init** (`Entry.rs`, `DevLog.rs`): `InitEager()` called at
  binary startup to create the session log before any panic. `BinarySignature()`
  fixed to correctly split PascalCase segments (`ElectronProfile` →
  `electron.profile`) so logs land in the correct app-data directory.
- **Static asset path resolution** (`Utilities.rs`): Extended to handle both
  `/Static/Application/` and `Static/Application/` forms. The webview WASM
  loader (`vscode-oniguruma` → `onig.wasm`) strips the leading slash before the
  path reaches the `file:read` IPC handler; without this fix `tokio::fs::read`
  returned ENOENT, breaking TextMate syntax highlighting.
- **macOS window dragging**: Removed `maximized(true)` from the window builder.
  Maximised macOS windows refuse all drag events regardless of
  `-webkit-app-region: drag` CSS. Replaced with `TitleBarStyle::Overlay` +
  `hidden_title(true)`. Windows/Linux continue using `decorations(false)`.
- **ENOENT ignore-list expanded** (`DevLog.rs`): Added `chatLanguageModels.json`,
  `configurationDefaultsOverrides`, window log files (`network.log`,
  `renderer.log`, `views.log`, `notebook.rendering.log`), `.copilot/agents`,
  `.vscode/tasks.json`, `/User/tasks.json`, `/User/mcp.json`,
  `vscode-chat-images`, `/output_<TIMESTAMP>` patterns.
- **Accidental `.bak` backup file removed**
  (`Source/Track/Effect/CreateEffectForRequest.rs.bak`).

---

### April 21, 2026

#### Added

- **Local VSIX installer — Atoms K2/K3** (`VsixInstaller.rs`): New module
  unpacks `.vsix` archives (ZIP with `extension/` prefix) into
  `~/.land/extensions`. Two-pass extraction reads the manifest before writing
  files; includes zip-slip protection. Produces `ExtensionDescriptionStateDTO`
  for the registry. Install/uninstall IPC handlers added in
  `WindServiceHandler/Extension.rs`. Notifies Cocoon via `$deltaExtensions` for
  hot-activation without reload; broadcasts `sky://extensions/installed` and
  `sky://extensions/uninstalled` for Wind sidebar refresh.
- **Kernel/minimal profile** (`ScanPathConfigure`): `LAND_SKIP_BUILTIN_EXTENSIONS`
  env var skips built-in extension scan paths while still scanning
  `~/.land/extensions` for VSIX-installed extensions.
- **`LAND_SPAWN_COCOON=false` flag**: Skips extension host spawn entirely —
  useful for Mountain-only integration tests and minimal shippable surface.
- **Environment forwarding to Cocoon**: `NODE_ENV`, `LAND_DEV_LOG`,
  `TAURI_ENV_DEBUG` now propagate to the Cocoon subprocess.
- **`.env.Land` bootstrap — Atom I5** (`AppLifecycle`): Loads environment from
  `.env.Land` at Mountain boot. Probes from cwd, parent dir, and repo-layout
  ancestors (up to 6 levels). Populates `Product*`, `Tier*`, `Network*` vars
  for standalone binary launches and forwards them to Cocoon for
  single-source-of-truth design.
- **Zombie-Cocoon prevention — Atom I6**: Pre-boot `SweepStaleCocoon()` TCP-
  probes port 50052, resolves owner via `lsof`, then SIGTERMs → SIGKILLs a
  stale process. Post-shutdown `HardKillCocoon()` force-terminates the child
  after the `$shutdown` gRPC attempt, preventing `EADDRINUSE` cascades on next
  launch.
- **Workspace activation fix — Atom I1** (`Window.navigate()` reload path):
  Before reload, `ApplicationState.Workspace` is mutated and
  `$deltaWorkspaceFolders` broadcast to Cocoon. Without this, Cocoon kept its
  pre-nav snapshot (0 folders) and `workspaceContains:*` activations never
  matched.
- **`security.workspace.trust.enabled=false` default** (`AppLifecycle`):
  Written to `User/settings.json` on first boot only if the key is absent.
  Without this, opening the Land repo as a workspace triggers VS Code's trust
  gate which marks built-in extensions as `DisabledByTrustRequirement` (because
  they ship inside the repo under `Element/Sky/Target/Static/Application/extensions/`).
- **BATCH-16 latency instrumentation** (`CreateEffectForRequest`): Timestamps
  added at dispatch-enter and body-start for `tree.register` latency analysis.
  `MountainVinegRPCService` also timestamps `grpc-recv` for hot-path RPCs,
  enabling 3-hop latency breakdown.

#### Changed

- **Cocoon health monitor** (`CocoonManagement.rs`): No longer floods logs with
  "No Cocoon process to monitor" after a crash — monitor exits quietly once the
  process is gone.
- **AppLifecycle fallback timers**: Timeout extended from 2 s → 8 s and 5 s →
  15 s with logging when Sky hasn't advanced phases; skipped if already advanced.
- **TerminalProvider**: Now captures and logs PID + exit status code on terminal
  exit.
- **DevLog timestamp**: Replaced manual UTC timestamp with `chrono::Local` to
  match VS Code's session log directory convention.
- **`extensions:getInstalled` handler** added to `WindServiceHandlers` for
  sidebar, with aux window stubs and shared `SessionTimestamp`.
- **gRPC connection logging**: Distinguishes expected startup race from real
  failures in Cocoon connection attempts — Atom I12.
- **Unknown command logging** — Atom L2: Distinguishes typos from
  registered-but-unimplemented channels.

#### Fixed

- **MIME type override** (`LocalhostPlugin`): Added explicit MIME overrides for
  JS/CSS/JSON/HTML/SVG assets to fix "not a valid JavaScript MIME type" errors
  in the webview.
- **Extension scan paths** (`ScanPathConfigure`): Removed `debug_only` gate,
  enabling repo-layout fallback in release builds.

---

### April 20, 2026

#### Added

- **Tier-gating system** (`Cargo.toml` feature flags, `build.rs`,
  `LandFixTier.rs`): Compile-time tier selection with runtime banner. New
  Cargo features for experimental tiers: `SharedMemory`, `Hyper`, `Ring`,
  `Layer4`, `Native`, etc. `EmitTierDefaults` ensures `env!()` resolves even
  without `.env.Land`. `LandFixTier` logs resolved tiers at boot.
- **Native file watcher — TierFileWatcherLayer4** (`FileWatcherProvider`):
  Backed by the `notify` crate (FSEvents / inotify /
  `ReadDirectoryChangesW`). Includes debouncing, glob→regex pattern filtering,
  and IPC forwarding to Cocoon as `$fileWatcher:event` notifications.
  Integrated into `CreateEffectForRequest` (`FileWatcher.Register/Unregister`).
- **Workspace folder runtime management**: `WorkspaceFolderCommand` Tauri
  commands (`MountainWorkspaceOpenFolder`, `List`, `CloseAll`); CLI parsing via
  `ParseWorkspaceFolders` (`--folder` flags, positional dirs,
  `LAND_WORKSPACE_FOLDER` env); boot seeding in `Entry::Fn` before extension
  activation.
- **DevLog file sink**: Logs to
  `~/Library/Application Support/<bundle>/logs/<timestamp>/Mountain.dev.log`.
  Enabled via `LAND_DEV_LOG_FILE=1` or auto-enabled in debug builds.
  Flush-per-line for `tail -f` compatibility.
- **Completed previously-stubbed IPC handlers**: `Terminal.Resize` (PTY master
  handle), `Clipboard.Read`/`Write` (`arboard` crate via `spawn_blocking`),
  `NativeHost.OpenExternal` (`open` crate with scheme validation), Webview
  operations (`sky://webview/*` Tauri events), Tree operations
  (register/unregister), `Task.Fetch`/`Execute`, `Authentication.GetSession`/
  `GetAccounts`, `Languages.GetAll`, `Debug.Stop`, `showOpenDialog`
  (`tauri-plugin-dialog`).
- `dependabot/fetch-metadata` bumped from 3.0.0 → 3.1.0.

---

### April 18, 2026

#### Changed

- **`dev_log!` macro formatting** (`f0449be`): All `dev_log!` invocations
  reformatted from single-line to multi-line across ~150 call sites in Air,
  ApplicationState, Binary, Environment, IPC, RPC, RunTime, Telemetry, and Vine
  modules. No functional changes.
- **Module encapsulation** (`8f12359`): Removed wildcard and specific `pub use`
  re-exports from `ApplicationState` internal modules and the Air module.
  Types must now be accessed via explicit module paths. Affected modules: Air
  (`AirClient`, `AirMetrics`, `AirStatus`), `ExtensionScanner`
  (`ScanAndPopulateExtensions`), `PathResolution`, `Persistence`, `Recovery`,
  `Serialization`.

#### Fixed

- **`.js.map` 204 handling** in `vscode-file` scheme handler for DevTools
  compatibility.
- **Extension scanner logging**: First-run non-existent directory is now
  `debug`-level instead of `warn`.
- **NLS placeholder resolution logging** added to extension scanner.

---

### April 17, 2026

#### Added

- **`vscode-file://` absolute OS path handling** (`336b21a`): Detects common
  macOS/Linux absolute path roots (`Volumes`, `Users`, `Library`, etc.) inside
  the `vscode-file://vscode-app/` URI and serves files directly from disk
  instead of resolving against `Sky/Target`. Fixes extension-contributed icon
  themes, grammars, and woff fonts.
- **NLS placeholder resolution in extension scanner**: Loads `package.nls.json`
  bundles and recursively replaces `%key%` placeholders in all string fields
  before parsing as `ExtensionDescriptionStateDTO`. Previously the Command
  Palette and menus displayed raw placeholder strings like `%command.clone%`.
- **FileSystem effects** (`CreateEffectForRequest`): `FileSystem.Stat`,
  `CreateDirectory`, `Delete`, `Rename`, `Copy` — completes the IPC bridge
  between Wind requests and Mountain's `FileSystemReader`/`Writer` traits.
- **Extension scanning instrumentation**: Comprehensive logging in
  `ScanAndPopulateExtensions` (inserted vs rejected counts), Scanner (directory
  entries, parse failures, missing `package.json`), `MountainEnvironment`
  (serialized non-null extensions), and Extension handler (payload sizes, empty
  result warnings). Native `setContext` command registered as no-op for VS Code
  extension compatibility.
- **Dev-only scan path `cfg` guards**: Fallback paths wrapped in
  `#[cfg(debug_assertions)]` to keep release builds from silently falling back
  to dev paths.

#### Changed

- **PascalCase naming migration — full RPC subsystem** (`5ba4958`): All
  remaining snake_case `.rs` files in the RPC layer renamed. Deprecated/unused
  service modules removed: `CommandService.rs`, `SecretStorageService.rs`,
  `WindowService.rs`, `WorkspaceService.rs`, `Telemetry/Metrics/mod.rs`,
  `Telemetry/Spans/mod.rs`. All `mod.rs` entries updated to explicit `#[path]`
  attributes.
- **PascalCase migration — LanguageFeature & Environment modules** (`9c0dce8`):
  `validation.rs` → `Validation.rs`, `hover.rs` → `Hover.rs`,
  `highlights.rs` → `Highlights.rs`, `completions.rs` → `Completions.rs`,
  `definition.rs` → `Definition.rs`, `references.rs` → `References.rs`,
  `tooltip.rs` → `Tooltip.rs`, `lifecycle.rs` → `Lifecycle.rs`,
  `configuration.rs` → `Configuration.rs`, `messaging.rs` → `Messaging.rs`.
- **PascalCase migration — initial pass** (`3461db4`): `CodeActions.rs`,
  `InvokeProvider.rs`, `ReadOperations.rs`, `WriteOperations.rs`,
  `EntryManagement.rs`, `MessageManagement.rs`, `EchoAction.rs`.
- **Dev node environment setup** (`62187df`): Extension scan paths extended —
  added `.app/Contents/Resources/app/extensions` (VS Code bundle convention)
  and `~/.land/extensions` (user-scope). Cocoon health monitor no longer loops
  after a crash.
- **Sky target path resolution corrected** (`3ba96f1`): Changed from
  `../../../Element/Sky/Target` (doubled `Element`) to `../../../Sky/Target` in
  `ScanPathConfigure.rs`, `AppLifecycle.rs`, and `InitializationData.rs`.

#### Fixed

- **Resource not found 404s** downgraded from `warn` to `info` in
  `FileSystem.ReadFile` — extensions probe for optional cache files on activate.
- **Empty `ticks` counter** removed from Cocoon health monitor.
- Remaining PascalCase module references (`ReadOperations`, `Messaging`) fixed.
- User extension path simplified to `.land/extensions` only.

---

### April 16, 2026

#### Added

- **Extension registration notification handlers** (`MountainVinegRPCService`):
  Three new notification handlers process messages from the Cocoon extension
  host — `window.showMessage` forwards info/warn/error messages to Sky via
  `sky://notification/show`; `registerCommand` stores proxied extension commands
  in `CommandRegistry` with `cocoon-main` sidecar identifier; provider
  registration fallback handles `register_hover_provider`,
  `register_completion_item_provider`, etc. for providers registered outside
  the typed RPC path.
- **Full language feature provider delegation** (`CocoonService`): 18 stub
  handlers replaced with actual delegations to `LanguageFeatureProviderRegistry`
  via `self.environment` — document highlights, symbols, workspace symbols,
  rename edits, document/range/on-type formatting, signature help, code lenses,
  folding ranges, selection ranges, semantic tokens (full), inlay hints, type
  hierarchy (super/subtypes), call hierarchy (incoming/outgoing), linked editing
  ranges.
- **Remaining 14 `FeatureMethods` implementations**
  (`LanguageFeatureProviderRegistry`): All remaining TODO stubs replaced —
  rename edits, document/workspace symbols, signature help, folding ranges,
  selection ranges, semantic tokens (full), inlay hints, type hierarchy
  (super/subtypes), call hierarchy (incoming/outgoing), linked editing ranges,
  on-type formatting. Each delegates to `FeatureMethods` →
  `ProviderLookup::get_matching_provider` → `invoke_provider`. `InsertText`
  handling in completion items fixed to properly extract string from JSON value.

#### Changed

- **`dev_log!` macro replaces `log` crate** (`b27a154`): All
  `log::{info, debug, error, warn, trace}` instances replaced with `dev_log!`
  macro accepting a category string across the entire Mountain codebase.
  Categories: `lifecycle`, `grpc`, `ipc`, `cocoon`, `extensions`, `config`,
  `model`, `storage`, `commands`, `output`, `terminal`.
- **Domain module split — Wind IPC** (`f8dd70e`): Monolithic
  `WindServiceHandlers` (167 KB) broken into 24 focused domain modules:
  Command, Configuration, Decoration, Environment, Extension, FileSystem,
  History, Keybinding, Label, Lifecycle, Model, NativeHost, Notification,
  Output, Progress, QuickInput, Search, Storage, Terminal, TextFile, Theme,
  WorkingCopy, Workspace.
- **Domain module split — Cocoon RPC** (`f8dd70e`): 2,800-line monolithic
  `CocoonService` split into 15 domain modules: Auth, Command, Debug, Extension,
  FileSystem, Output, Provider, SCM, Save, Secret, Task, Terminal, TreeView,
  Window, Workspace. Language feature provider handlers consolidated in
  `Provider.rs`.
- **Vine gRPC binding** changed from IPv6 `[::1]` to IPv4 `127.0.0.1`.
- **Startup extension activation trigger** (`$activateByEvent("*")`) added
  after Cocoon handshake.
- **`node_modules` path resolution** in `vscode-file` scheme handler corrected
  from `Static/node_modules/` to `Static/Application/node_modules/`.
- **Filesystem asset fallback** added for dev mode when assets aren't embedded.
- **Tauri CSP** updated to include `vscode-file:` protocol in `connect-src`.
- **`serde_json::json` macro import** added to `MountainVinegRPCService` for
  structured JSON payload construction in the Vine protocol service layer.

#### Fixed

- **`MutexGuard` Send fix** (`FeatureMethods.rs`): Lock scope narrowed to
  prevent `MutexGuard` from being held across `.await` point.
- **Duplicate `dev_log` imports** removed from `AdvancedFeatures.rs` and
  `WindAdvancedSync.rs`.
- **`TraceLog.rs`** syntax error and **`ConfigurationInitialize.rs`** broken
  imports resolved.
- **Provider registration** fixed to use `self.RunTime.Environment` instead of
  `self.Environment` for correct runtime state access.
- **Command registration locking** corrected — `CommandRegistry` now properly
  locked before insertion, using correct `CommandHandler::Proxied` structure
  with `SideCarIdentifier` and `CommandIdentifier` fields.

---

### Earlier v2.1 (pre-April 16)

- 14 language feature provider methods in `FeatureMethods.rs` (354 lines):
  `renameEdits`, `documentSymbols`, `workspaceSymbols`, `signatureHelp`,
  `foldingRanges`, `selectionRanges`, `semanticTokensFull`, `inlayHints`,
  `typeHierarchySupertypes`, `typeHierarchySubtypes`, `callHierarchyIncoming`,
  `callHierarchyOutgoing`, `linkedEditingRanges`, `onTypeFormattingEdits`
- 18 CocoonService stub handlers replaced with Tauri event implementations
  (`sky://` patterns for progress, webview, terminal, tree view, SCM, debug,
  tasks, auth, config, external)
- Extension registration notification handlers in `MountainVinegRPCService`:
  `window.showMessage`, `registerCommand`, provider registration fallback
- 5 `ProviderType` enum variants in Common: Task, Authentication, TreeView,
  SourceControl, DebugAdapter
- Startup extension activation trigger (`$activateByEvent("*")`) after Cocoon
  handshake
- `dev_log!` macro replaced `log` crate across 170 files with categorised tags:
  lifecycle, grpc, ipc, cocoon, extensions, config, model, storage, commands,
  output, terminal
- Vine gRPC binding changed from IPv6 `[::1]` to IPv4 `127.0.0.1`
- `MutexGuard` Send fix in `FeatureMethods` (scope narrowing across `.await`)
- `TraceLog.rs` syntax error, `ConfigurationInitialize.rs` broken imports fixed
- `AdvancedFeatures.rs` + `WindAdvancedSync.rs` duplicate `dev_log` imports fixed
- `CompletionItem` DTO missing `documentation` field added
- Tauri CSP updated to include `vscode-file:` in `connect-src`
- `node_modules` path resolution in `vscode-file` scheme handler fixed
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

- RPC error response: string → proper RpcError struct with JSON-RPC code -32601
- DTO deserialization: InputBoxOptionsDTO, OpenDialogOptionsDTO,
  SaveDialogOptionsDTO corrected from `>(obj.clone())` to
  `>(Value::Object(obj.clone()))`
- Vine protobuf field name alignment: RegisterTreeViewProviderRequest
  display_name → extension_id, GitExecRequest repository/cwd →
  repository_path/args

## [v1.3] - Q4 2025: Dependency Maintenance

### Added

- `gen/schemas/macOS-schema.json` - platform-specific capabilities for tray,
  menu bar, dock integration

### Changed

- actions/cache 4.3.0 → 5.0.1
- actions/checkout 5.0.0 → 6.0.1
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
- Module naming: handlers → Command, app_state → ApplicationState
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

- Deprecated `Source/app_state/` → `Source/ApplicationState/`
- Cargo.toml: removed 45 redundant entries, added 17 new entries

## [v0.2] - Q4 2024: Architecture Solidification

### Added

- `gen/android/` - full Android Gradle project scaffold:
  - AndroidManifest.xml, MainActivity.kt
  - RustPlugin.kt, BuildTask.kt (Rust → Android cross-compilation)
  - Resources: drawable, mipmap (hdpi through xxxhdpi), layout, values
- `gen/schemas/mobile-schema.json` (11,162 lines)
- `gen/schemas/android-schema.json` (11,162 lines)
- Cargo.toml feature flags: AirIntegration, ExtensionHostCocoon, MistNative,
  Debug, grove, cocoon, terminals, debug-protocol, scm-support, Telemetry

## [v0.1] - Q3 2024: Rapid Development

### Changed

- Schema reduction: 8,467 → ~1,000 lines in ACL manifests (78% reduction)
- Removed bloated capabilities; kept only essential Tauri permissions
- Binary function refactoring: `Source/Fn/Binary.rs` 106 → ~20 lines
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
