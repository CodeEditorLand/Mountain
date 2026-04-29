# Changelog - Mountain

Mountain is the native Rust backend that hosts our Tauri app, the gRPC
server we expose to Cocoon, and the runtime that owns every workbench
service routed through Rust. This file records what we built in our
voice, version by version. Format adapted from
[Keep a Changelog](https://keepachangelog.com/).

## [v2.2] - Bundled-Electron Profile: Correctness Pass

We brought the bundled-electron profile to correctness parity with the
unbundled path and started landing the substrate fixes that take us
past it. Headline number: we drove the Cocoon-facing circuit-breaker
cascade from **17,134 rejections per workspace session** to **zero**.

### Added

- **JSON-RPC code `-32004`** for benign 404s. Mountain's
  `LooksLike404` predicate in `Vine/Server/MountainVinegRPCService.rs`
  now matches `no such file or directory`, `entity not found`,
  `os error 2`, `path is outside of the registered workspace`,
  `permission denied for operation`, and `workspace is not trusted`,
  so Rust's `std::io::Error` formatting stops tripping the breaker
  on extension probes. We stamp `-32004` for these so the client
  classifies on code first.
- **Three-shape `GetURLFromURIComponentsDTO`** in
  `Environment/Utility/UriParsing.rs`: accepts a plain string,
  `{external}` object, or `{scheme,path}` components. Diagnostic
  publishes from rust-analyzer stop dropping when the URI shape
  varies between code paths.
- **Deferred-no-op file watching** in `Environment/FileWatcherProvider.rs`.
  Watch on absent path now records the entry without a live OS watcher
  and returns `Ok` so the optional config dirs every workspace probes
  (`~/.roo/skills-*`, `.vscode/settings.json` in fresh workspaces) stop
  counting against the breaker.
- **`FileWatcher.Register` handle accepts `Value::Number`** in
  `Track/Effect/CreateEffectForRequest/FileWatcher.rs`. Cocoon's
  numeric `NextProviderHandle()` no longer collapses 136 watcher
  handles to a single empty-string entry.
- **Canonical-path comparison** in
  `Environment/Utility/PathSecurity.rs::IsPathAllowedForAccess`. We
  try both raw and canonical paths on each side so workspace folders
  mounted under `/Volumes/...` on macOS stop rejecting their own
  descendants. Reject details now log under the `vfs` tag.
- **`Identifier:String` field** on `SourceControlManagementProviderDTO`
  in Common, threaded through `Environment/SourceControlManagementProvider.rs`
  so SCM replay re-emits with the original provider id (`git`,
  `github`, `hg`, …) instead of a hardcoded fallback.
- **`sky:replay-events` IPC handler** in
  `IPC/WindServiceHandlers/mod.rs`. On renderer-bridge install we
  drain tree-view registrations, SCM providers, extension-registered
  commands, active terminals + their per-PTY ring buffer, and re-emit
  each on its canonical channel. Closes the boot-window event-race that
  silently dropped startup events between Mountain emit and renderer
  listen.
- **64 KB per-terminal ring buffer** in `Environment/TerminalProvider.rs`.
  PTY reader appends every chunk to the buffer before emitting; replay
  drains it on listener install so the shell's first prompt + zsh MOTD
  reach xterm even when the renderer bundle takes ~1.5 s to boot.
- **`LogSkyEmit` instrumentation** on the SCM register emit
  (`Vine/Server/Notification/RegisterScmProvider.rs`). Visibility for
  the prior fire-and-forget path that dropped silently.

### Changed

- **`tree:getChildren` timeout 5000 ms → 1500 ms** in
  `IPC/WindServiceHandlers/mod.rs`. Slow extensions stop eating five
  seconds of perceived UI responsiveness; we fall back to empty array
  and let the workbench schedule its own retry.

## [v2.1] - Full Workbench Lift, Search End-to-End, SCM/Debug/Custom-Editor

We brought the bundled VS Code workbench up in our Tauri webview
end-to-end. Mountain compiles clean, Cocoon attaches over gRPC, all
113 of our test extensions scan, the workbench renders, and we kept
chasing gaps until search, SCM, debug, and custom editors all worked.

### Added

- **24-module split of `WindServiceHandlers`** and **15-submodule
  split of `CocoonService`** so each domain (Commands, Configuration,
  Extensions, FileSystem, Model, NativeHost, Navigation, Output,
  Search, Storage, Terminal/PTY, UI, Utilities, …) lives in its own
  file. Easier to grep, easier to land per-domain fixes.
- **`dev_log!` macro** rolled across 170 files. Replaces the old `log`
  crate with a tag-filtered stream - `Trace=terminal,exthost ./Mountain`
  filters live. 38 granular tags (`terminal`, `extensions`, `search`,
  `themes`, `window`, `nativehost`, `clipboard`, `commands`, `model`,
  `output`, `notification`, `progress`, `quickinput`, `workingcopy`,
  `workspaces`, `keybinding`, `label`, `history`, `decorations`,
  `textfile`, `update`, `encryption`, `menubar`, `url`, `grpc`,
  `cocoon`, `bootstrap`, `preload`, …).
- **`Trace=short` mode**: compresses module targets
  (`D::Binary::Main::Entry` → `Entry`), aliases verbose app-data path
  to `$APP`, and dedupes consecutive identical messages with `(xN)`
  suffix.
- **18 CocoonService handlers + 14 language-feature RPC methods**.
  Stub implementations replaced with real delegations to
  `LanguageFeatureProviderRegistry` covering document highlights /
  symbols / workspace symbols, rename edits, formatting (document /
  range / on-type), signature help, code lenses, folding ranges,
  selection ranges, semantic tokens, inlay hints, type hierarchy,
  call hierarchy, linked-editing ranges.
- **Comprehensive `CreateEffectForRequest` effect creators** for
  Authentication, Clipboard, Commands, Configuration, Debug,
  Diagnostics, Documents, FileSystem, FileWatcher, Git, Keybinding,
  Languages, NativeHost, SCM, Search, Secrets, StatusBar, Storage,
  Task, Terminal, TreeView, UserInterface, Webview, WindowUI,
  Workspace.
- **PostHog plugin** (`Binary/Build/PostHogPlugin.rs`) for debug-build
  analytics: lifecycle events, IPC commands, errors. API key, host,
  and enable flag baked at compile time via `build.rs`'s
  `PropagatePostHogSentinel()`.
- **OTLP proxy** in `LocalhostPlugin`. Forwards `/v1/traces` requests
  to a local OTLP collector at `127.0.0.1:4318` via raw TCP so the
  renderer's `OTELBridge.ts` can send telemetry without CORS issues.
- **`NodeResolver` module** (`ProcessManagement/NodeResolver.rs`).
  Eight-step ladder: `Pick` env → shipped runtime → fnm / volta /
  asdf / nvm → Homebrew → PATH fallback. Verifies version against
  `Require` (default 20) and warns on downgrade.
- **`UriComponents` helper module** (`Environment/UriComponents/`).
  Centralises every URI payload sent to the renderer with the required
  `$mid: 1` marshalling marker. Exposes `FromFilePath`, `FromUrl`,
  `StampMidUri`, `Normalize`.
- **Local VSIX installer** (`Environment/VsixInstaller.rs`). Two-pass
  extraction reads the manifest before writing, includes zip-slip
  protection, unpacks into `~/.land/extensions`, then notifies Cocoon
  via `$deltaExtensions` for hot activation without a workbench
  reload.
- **`extensions:getManifest` IPC handler**. Reads
  `extension/package.json` straight from a `.vsix` archive without
  disk extraction so the Install-from-VSIX preview dialog works.
- **`extensions:scanSystemExtensions` / `scanUserExtensions` routing**
  with explicit type filter so VSIX-installed extensions show up under
  Installed with an Uninstall action.
- **`extensions:resetPinnedStateForAllUserExtensions`**.
- **Native file watcher** (`Environment/FileWatcherProvider.rs`).
  Backed by the `notify` crate. Includes debouncing, glob-to-regex
  pattern filtering, and IPC forwarding to Cocoon as
  `$fileWatcher:event`.
- **`BuildInitialUrl`** reads `~/.land/workspaces/RecentlyOpened.json`
  and passes `?folder=<path>` to the initial webview URL, skipping the
  destructive reload-on-Welcome.
- **`ParseWorkspaceFolders`** CLI parsing: `--folder` flags,
  positional dirs, `Open` env.
- **DevLog file sink**: `~/Library/Application Support/<bundle>/logs/
  <timestamp>/Mountain.dev.log`. Enabled via `Record=1`.
- **`DevLog::InitEager()`** at binary startup so a session log exists
  before any panic, preserving post-mortem evidence.
- **Atomic shutdown guard** in `Binary/Main/Entry.rs`. Tauri
  re-delivers `ExitRequested { code: Some(0) }` after `app_handle.exit(0)`
  - without the guard the shutdown task ran twice, tried to SIGKILL
  an already-dead Cocoon, and logged spurious tcp-connect errors.
- **Pre-boot zombie sweep** (`SweepStaleCocoon`). TCP-probes port
  50052 and SIGTERMs/SIGKILLs a stale Cocoon process so successive
  boots don't fail with `EADDRINUSE`. Paired with post-shutdown
  `HardKillCocoon`.
- **Tier-gating system** (`Cargo.toml` feature flags + `build.rs`):
  compile-time tier selection with a runtime banner so we know which
  layer is active per profile (`TierFileSystem=Layer2` etc.).
- **Five extension-path env vars** (`Skip`, `Ship`, `Lodge`,
  `Extend`, `Probe`) for switching between bundled, user, and
  test-extension layouts.
- **Navigation history state** with `goBack`, `goForward`,
  `canGoBack`, `canGoForward`, `push`, `clear`, `getStack` IPC
  handlers.
- **Label resolution IPC handlers** (`getUri`, `getWorkspace`,
  `getBase`) for human-readable display labels.
- **Text model registry IPC handlers** (`open`, `close`, `get`,
  `getAll`, `updateContent`).
- **DualTrack env knobs** (`LAND_DEFER_RUST*`): per-method,
  per-domain, or global Rust deference for A/B testing Mountain vs
  Cocoon paths.
- **macOS window dragging via overlay title-bar**:
  `TitleBarStyle::Overlay` + `hidden_title(true)`. Removed the
  `maximized(true)` that blocked drag.

### Changed

- **`SkyEvent` typed constants** replace hardcoded `"sky://output/*"`
  string literals. Source of truth in `CommonLibrary::IPC::SkyEvent`.
- **Vine protocol PascalCase** end-to-end. All RPC methods renamed
  from `snake_case` to `PascalCase` (e.g. `initial_handshake` →
  `InitialHandshake`); `vine.rs` regenerated.
- **Vine gRPC binding** moved from IPv6 `[::1]` to IPv4 `127.0.0.1`
  for cross-platform reach.
- **PascalCase rename across entire RPC subsystem**: every snake_case
  `.rs` file renamed; deprecated service modules removed; module-level
  re-exports cleaned so import paths spell out where each symbol
  lives.
- **DTO serialization**: removed `skip_serializing_if = "String::is_empty"`
  from `Name`, `Version`, `Publisher` on `ExtensionDescriptionStateDTO`.
  Trusted-publishers migration in the workbench was crashing on
  `manifest.publisher.toLowerCase()` when the field was absent.
- **`posthog-rs` 0.5 API** migration: `api_endpoint()` → `host()`.
- **`sha2` 0.11 migration**: `LowerHex` impl → `hex::encode()` for our
  session-id hash.
- **Resource-not-found 404s** downgraded from `warn` to `info`. The
  benign-ENOENT ignore list expanded for `chatLanguageModels.json`,
  window log files, `.copilot/agents`, `.vscode/tasks.json`,
  `/User/mcp.json`, `vscode-chat-images`, `/output_<TIMESTAMP>`.
- **Cocoon health monitor** stops flooding logs after a crash; AppLifecycle
  fallback timers extended (2 s → 8 s, 5 s → 15 s).
- **Workspace folder API** sync → async. `GetFolders().await` replaced
  with `GetWorkspaceFolders()`; add/remove use direct manipulation via
  `GetWorkspaceFolders` / `SetWorkspaceFolders`.
- **Vine protobuf field names** aligned: `RegisterTreeViewProviderRequest`
  `display_name` → `extension_id`; `GitExecRequest` `repository`/`cwd`
  → `repository_path`/`args`.

### Fixed

- **Extension type filtering** in `IPC/WindServiceHandlers/Extensions.rs`.
  `getInstalled(type?)` now respects the optional filter (0 = System,
  1 = User) so VSIX-installed extensions appear under Installed.
- **Post-install activation burst**: after `$deltaExtensions` adds an
  extension to Cocoon's registry we fire `onStartupFinished`. Without
  this, extensions with that activation event registered but never
  activated until the next workbench reload.
- **Extension manifest passthrough**: 14 fields added to
  `ExtensionDescriptionStateDTO` (`Categories`, `DisplayName`,
  `Description`, `Keywords`, `Repository`, `Bugs`, `Homepage`,
  `License`, `Icon`, `AiKey`, `ExtensionKind`, `Capabilities`,
  `ExtensionDependencies`, `ExtensionPack`).
- **Static asset path resolution** in `Utilities.rs`. Accepts both
  `/Static/Application/` (absolute) and `Static/Application/`
  (relative). The WKWebView WASM loader strips the leading slash
  before reaching `file:read`, which used to fail with `ENOENT` and
  broke TextMate syntax highlighting for every grammar.
- **`vscode-file://` absolute OS-path handling**: detects macOS/Linux
  absolute path roots inside `vscode-file://vscode-app/` and serves
  files directly from disk. Fixes extension-contributed icon themes,
  grammars, woff fonts.
- **NLS placeholder resolution** in the extension scanner: loads
  `package.nls.json` and recursively replaces `%key%` placeholders
  before parsing the manifest.
- **`MutexGuard` Send fix** in `FeatureMethods.rs`: lock scope
  narrowed across `.await` points.
- **MIME-type override** in `LocalhostPlugin`: explicit MIME for
  JS / CSS / JSON / HTML / SVG assets.
- **Provider selector format** standardised to `[{"language": ...}]`;
  `SideCarIdentifier` always routes to `"cocoon-main"`.
- **Language-inference fallback** in `ProviderLookup`: derives
  language from file extension (`.rs` → rust, `.ts` → typescript,
  …) when the document isn't in open state.
- **Storage `DeleteItem`** replaced with
  `UpdateStorageValue(true, Key, None)`.
- **`QuickInput` field naming**: `Placeholder` → `PlaceHolder`.
- **Dialog options DTO**: wrapped in `Base` field with
  `DialogOptionsDTO` containing `Title`. Path strings via `.display()`.
- **First-run scanner directory** now `debug`-level instead of `warn`.
- **`.js.map` 204 handling** in the `vscode-file` scheme handler.

## [v2.0] - Editor Launch

We took Mountain from "compiles" to "boots and renders" and put every
core service in place to back the workbench.

### Added

- **`Source/Binary/` modular startup**:
  - `Tray/` (`SwitchTrayIcon`, `EnableTray`).
  - `Shutdown/` (`SchedulerShutdown`, `RuntimeShutdown`).
  - `Service/` (`VineStart` for gRPC on 50051/50052, `CocoonStart`,
    `ConfigurationInitialize`).
  - `Register/` (`AdvancedFeaturesRegister`, `IPCServerRegister`,
    `StatusReporterRegister`, `WindSyncRegister`).
  - `Initialize/` (`RuntimeBuild`, `StateBuild`, CLI argument parser,
    dynamic port selection).
- **20+ Tauri commands** exposed: workbench/desktop configuration
  query, update-subscription endpoint, tray-icon switch, IPC status
  query, performance statistics, message-reception endpoint,
  generic-IPC-method invocation, document synchronisation,
  collaboration session, status history.
- **TreeView foundation**: stub interaction handlers, state
  persistence skeleton, Tauri event propagation. Concrete
  `OnTreeNodeExpanded` / `OnTreeSelectionChanged` followed in v2.0
  with `ApplicationState` locks and event broadcast.
- **Hover feature** (`Source/Command/Hover/`): `Fn.rs` (164 lines)
  with hover-request handling.
- **Debug state** (`ApplicationState/State/FeatureState/Debug/
  DebugState.rs`, ~155 lines) with
  `DebugConfigurationProviderRegistration` and
  `DebugAdapterDescriptorFactoryRegistration`.
- **Vine gRPC protocol expansion**: full coverage for language
  features, window operations (quick-pick, input-box, progress,
  webview messaging, external URI), file system, output channels,
  tasks, authentication, extensions, terminal resize, configuration.
- **Generic request router** (`process_mountain_request`): routes
  `fs.*` and `commands.execute` methods.
- **Terminal I/O, secret storage, UI dialogs**: `terminal_input`,
  `close_terminal`, `get_secret` / `store_secret` / `delete_secret`
  via OS keychain, `show_quick_pick`, `show_input_box`.
- **Output channel events + configuration retrieval** with Tauri
  emit fan-out to the renderer.
- **`/Static/Application` path mapping** in `normalize_uri_path`,
  pre-creating system-extensions directory to avoid `ENOENT` on
  first scan.
- **351 Rust files** across the workspace, 70+ gRPC RPCs, 24 IPC
  domain modules.
- **TLS, AES-256-GCM, OpenTelemetry, PostHog** initial plumbing.

### Changed

- **`dev_log!` macro replaces `log` crate** project-wide.
- **Domain-module split**: 167 KB monolithic `WindServiceHandlers` →
  24 modules; 2,800-line `CocoonService` → 15 submodules.
- **Thread safety**: `static mut` → `RwLock` in `Scheme.rs` (829
  lines), `ServiceRegistry.rs` (820 lines), `CertificateManager.rs`
  (395 lines), `TlsCommands.rs` (184 lines). Cleaned 150+ unused
  imports across 40+ provider files.
- **Binary entry point** moved from `Library.rs` into
  `Source/Binary.rs`. Library API and binary configuration now
  separable.

### Fixed

- **RPC error response**: string → proper `RpcError` struct with
  JSON-RPC code `-32601`.
- **DTO deserialisation** for `InputBoxOptionsDTO`,
  `OpenDialogOptionsDTO`, `SaveDialogOptionsDTO`.
- **`CompletionItem` DTO**: missing `documentation` field added.

## [v1.3] - Dependency Maintenance

We held steady. `gen/schemas/macOS-schema.json` added for tray /
menu bar / dock integration. `actions/cache` 4.3.0 → 5.0.1 and
`actions/checkout` 5.0.0 → 6.0.1 in CI; regular `.github/Update.md`
auto-increments.

## [v1.2] - Full-Stack Integration

We standardised error handling across `SourceControlManagement`,
`DocumentProvider`, and `TerminalProvider`; consolidated sidecar
naming; added the `22NodeVersion` label to build artifacts so the
Node.js sidecar binary version is visible from the file name; and
standardised Windows installer naming across debug and release
profiles.

## [v1.1] - Architecture Buildout

The complete-architecture quarter - 185 commits.

### Added

- **`Source/Command/`**: `CommandRegistry`, `Keybinding`,
  `LanguageFeature`, `SourceControlManagement`, `TreeView`,
  `Bootstrap`, `Hover`.
- **`Source/Environment/`**: 14 provider implementations
  (`Configuration`, `Document`, `FileSystem`, `Output`, `Search`,
  `Secret`, `StatusBar`, `Storage`, `Terminal`, `TreeView`,
  `UserInterface`, `Webview`, `Workspace`, `Utility`).
- **`Source/ProcessManagement/`** (`CocoonManagement`,
  `InitializationData`).
- **`Source/RunTime/`** - `ApplicationRunTime` executable lifecycle.
- **`Source/Track/`** - request routing + effect system
  (`DispatchLogic`, `EffectCreation`).
- **`Source/Update/`**, **`Source/Workspace/WorkSpaceFileService`**,
  **`Source/FileSystem/FileExplorerViewProvider`**.
- **`Source/Vine/Server/`** (`MountainVinegRPCService`, `Initialize`).
- **`Source/Vine/Generated/vine_ipc.rs`** (1,714 lines auto-generated
  from `Vine.proto`).
- **Node.js sidecar bundled** alongside the binary in `Target/debug/`
  and `Target/release/`.
- **Windows artefacts**: `.exe`, `.msi`.

### Changed

- Error handling unified via `CommonError` + `MapLockError` pattern.
- Module renames: `handlers` → `Command`, `app_state` →
  `ApplicationState`.
- README rewritten with architecture overview (76 lines);
  `docs/Deep Dive.md` rewritten (182 lines).
- License transitioned to CC0 1.0 Universal.

## [v1.0] - Integration Phase

### Added

- **`Source/ApplicationState/` reorganization**:
  `Internal/{Persistence, Recovery, TextProcessing, PathResolution,
  ExtensionScanner, Serialization}`, `DTO/` with 12 classes
  (`DocumentStateDTO`, `TerminalStateDTO`, `TreeViewStateDTO`,
  `WindowStateDTO`, …), `State/` with feature-organized state.
- **`.github/workflows/Auto.yml`** (68 lines) for the automated
  update/push CI.
- **`Knowledge.dot`** + **`Knowledge.svg`** module dependency graph.

### Changed

- Deprecated `Source/app_state/` → `Source/ApplicationState/`.
- `Cargo.toml`: removed 45 redundant entries, added 17 new ones.

## [v0.2] - Architecture Solidification

We brought up the Android scaffolding. `gen/android/` includes
`AndroidManifest.xml`, `MainActivity.kt`, `RustPlugin.kt`,
`BuildTask.kt` (Rust → Android cross-compilation), and resources for
drawable / mipmap (hdpi → xxxhdpi) / layout / values.
`gen/schemas/{mobile,android}-schema.json` (11,162 lines each)
landed alongside, and we added the `Cargo.toml` feature-flag family
(`AirIntegration`, `ExtensionHostCocoon`, `MistNative`, `Debug`,
`grove`, `cocoon`, `terminals`, `debug-protocol`, `scm-support`,
`Telemetry`).

## [v0.1] - Rapid Development

We did our biggest schema reduction in the same window: 8,467 →
~1,000 lines in ACL manifests (78 % cut). Removed bloated
capabilities; kept only essential Tauri permissions. Refactored
`Source/Fn/Binary.rs` from 106 to ~20 lines and added
`Source/Fn/Binary/Notes.md` (91 lines) as architectural reference.

## [v0.0] - Project Inception

We relocated the project from `Editor/editor/src-tauri/` to the
monorepo root. `Source/Library.rs` became the single Tauri entry
point. `tauri.conf.json` (94 lines) covered Windows NSIS, macOS DMG,
and `.deb` bundle targets; `build.rs` integrated the Tauri build
system. CI landed via `.github/workflows/{Rust,GitHub}.yml` plus
`dependabot.yml`. App icons (PNG, ICNS, ICO, Windows tiles),
`capabilities/` ACL manifest, and `gen/schemas/{desktop,windows}-schema.json`
all shipped in this window.

### Initial dependencies

- **Core**: `tauri`, `tokio`, `serde`, `serde_json`, `tonic` (gRPC).
- **Plugins**: `tauri-plugin-{dialog,fs,localhost,log}`.
- **Serialisation**: `prost` (protobuf), `bincode`.
- **Crypto**: `sha2`, `md5`, `ring`, `rcgen`, `p256`, `x509-parser`,
  `rustls`, `pem`.
- **Network**: `tokio-tungstenite`, `http`, `url`.
- **Terminal**: `portable-pty`.
- **Process**: `sysinfo`, `hostname`.
