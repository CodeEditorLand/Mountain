# **Mountain**&#x2001;⛰️

<table>
	<tr>
		<td>
			<a href="https://GitHub.Com/CodeEditorLand/Mountain" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/last-commit/CodeEditorLand/Mountain?label=Update&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/last-commit/CodeEditorLand/Mountain?label=Update&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/last-commit/CodeEditorLand/Mountain?label=Update&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Update" title="Update" />
				</picture>
			</a>
			<br />
			<a href="https://GitHub.Com/CodeEditorLand/Mountain" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/issues/CodeEditorLand/Mountain?label=Issue&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/issues/CodeEditorLand/Mountain?label=Issue&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/issues/CodeEditorLand/Mountain?label=Issue&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Issue" title="Issue" />
				</picture>
			</a>
		</td>
		<td>
			<a href="https://github.com/CodeEditorLand/Mountain" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/stars/CodeEditorLand/Mountain?style=flat&label=Star&logo=github&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/stars/CodeEditorLand/Mountain?style=flat&label=Star&logo=github&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/stars/CodeEditorLand/Mountain?style=flat&label=Star&logo=github&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Star" title="Star" />
				</picture>
			</a>
			<br />
			<a href="https://GitHub.Com/CodeEditorLand/Mountain" target="_blank">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/github/downloads/CodeEditorLand/Mountain/total?label=Download&color=black&labelColor=black&logoColor=white&logoWidth=0" />
					<source media="(prefers-color-scheme: light)" srcset="https://img.shields.io/github/downloads/CodeEditorLand/Mountain/total?label=Download&color=white&labelColor=white&logoColor=black&logoWidth=0" />
					<img src="https://img.shields.io/github/downloads/CodeEditorLand/Mountain/total?label=Download&color=black&labelColor=black&logoColor=white&logoWidth=0" alt="Download" title="Download" />
				</picture>
			</a>
		</td>
	</tr>
</table>

The Native `Rust`/`Tauri` Desktop Shell for Land&#x2001;🏞️

> **The RAM tax is not optional.** `VS Code` with a medium project: 500 MB to
> 1.5 GB of RAM. Three open windows means three `Chromium` renderer processes,
> each carrying a full heap. Every OS interaction crosses a serialized JSON IPC
> pipe.

_"Where `Electron` takes 200 ms to open a dialog, `Mountain` takes 2."_

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Mountain/tree/Current/LICENSE)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Crates.io](https://img.shields.io/crates/v/Mountain.svg)](https://crates.io/crates/Mountain)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Rust Version](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org/)
[<img src="https://editor.land/Image/Tauri.svg" width="14" alt="Tauri" />](https://tauri.app/) [![Tauri Version](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![gRPC](https://img.shields.io/badge/gRPC-tonic-blueviolet.svg)](https://github.com/hyperium/tonic)

**[Rust API Documentation](https://rust.documentation.mountain.editor.land/)**&#x2001;📖

---

## Overview

**Mountain** is the foundational `Rust`/`Tauri` backend for the
**Land**&#x2001;🏞️ Code Editor. It implements the abstract service
`trait`s defined in the `Common`&#x2001;🧑🏻‍🏭 crate, providing native-speed
implementations for filesystem I/O, process management, secure storage, and
more. It manages the application lifecycle, orchestrates native OS operations,
launches and communicates with the `Cocoon`&#x2001;🦋 (`Node.js`)
extension host sidecar via `gRPC`&#x2001;🌿, and serves as the backend for the
`Wind`&#x2001;🍃 layer through `Tauri` commands and events.

`Electron` ships an entire `Chromium` renderer per window, each carrying a full
heap. Every OS interaction crosses a serialized JSON IPC pipe. **Mountain**
replaces that with a single `Rust` binary backed by `Tauri` - OS-level native
calls, a work-stealing scheduler (`Echo`&#x2001;📣), and a `gRPC`-based sidecar
protocol. The result: dialogs open in 2 ms instead of 200, and memory usage is a
fraction of `Electron`'s.

**Mountain is engineered to:**

1. **Be the Native Core** - Act as the primary `Rust` application, leveraging
   `Tauri` to create a lightweight, cross-platform windowing and `WebView` host.
2. **Provide High-Performance Services** - Implement the abstract service
   `trait`s defined in the `Common`&#x2001;🧑🏻‍🏭 crate, offering
   native-speed implementations for filesystem I/O, process management, secure
   storage, and more.
3. **Orchestrate Sidecars** - Reliably launch, manage, and communicate with the
   `Cocoon`&#x2001;🦋 (`Node.js`) extension host sidecar via a robust
   `gRPC`&#x2001;🌿 interface.
4. **Power the User Interface** - Serve as the backend for the
   `Wind`&#x2001;🍃 layer, responding to requests via `Tauri` commands
   and pushing state updates via `Tauri` events.

---

## Key Features&#x2001;⛰️

**Declarative Effect System** - Built on a `Rust` `ActionEffect` system defined
in the `Common`&#x2001;🧑🏻‍🏭 crate. Business logic is described as
declarative, composable effects, executed by a central `ApplicationRunTime`.
Every incoming request (from `Wind`&#x2001;🍃, `Cocoon`&#x2001;🦋, or internal
triggers) maps to an effect that the runtime executes with retry and timeout
semantics.

**`gRPC`-Powered IPC** - Hosts a `tonic`-based `gRPC` server (`Vine`&#x2001;🌿)
on port 50052 to provide a strongly-typed, high-performance communication
channel for the `Cocoon`&#x2001;🦋 extension host. All extension host ↔
native communication flows through this channel.

**Centralized State Management** - Utilizes a thread-safe, `Tauri`-managed
`ApplicationState` as the single source of truth for the entire application's
state, from open documents to provider registrations. State is persisted to disk
via `Memento` save/load with corruption recovery and validation.

**Native PTY Management** - Implements a full-featured integrated terminal
service by spawning and managing native pseudo-terminals (`PTY`) using the
`portable-pty` crate, with shell integration, environment collection, and
terminal state serialization for hot reload.

**Secure Storage Integration** - Leverages the native OS keychain via the
`keyring` crate to securely store sensitive data like authentication tokens, API
keys, and extension secrets.

**Robust Command Dispatching** - A central `Track` dispatcher intelligently
routes all incoming requests from `Wind`&#x2001;🍃, `Cocoon`&#x2001;🦋, and
`Webview` panels to the appropriate native `Environment` provider or
`ActionEffect`, with permission validation and audit logging.

**Extension Management** - Discovery, manifest parsing, and VSIX installation
for VS Code-compatible extensions. Includes NLS (National Language Support)
resolution and default configuration merging.

**Over-the-Air Updates** - Integrated update system using the
`Air`&#x2001;🪁 daemon for checking, downloading, and applying
application updates with progress reporting and automatic restart.

---

## Core Architecture Principles&#x2001;🏗️

| Principle                       | Description                                                                                                                                   | Key Components                           |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| **Implementation of Contracts** | Implement the abstract service `trait`s from `Common`&#x2001;🧑🏻‍🏭, providing concrete logic for the application's architecture.                 | `Environment/*` providers                |
| **Separation of Concerns**      | Isolate service logic into distinct `Environment` provider modules, each responsible for a specific domain (e.g., `FileSystem`, `Documents`). | `Environment/*`, `Command/*`             |
| **Declarative Logic**           | Express complex operations as `ActionEffect`s, executed by `ApplicationRunTime` - composable, testable, and robust.                           | `RunTime/*`, `Track/*`, `Common`         |
| **Centralized State**           | Maintain a single, thread-safe `ApplicationState` struct managed by `Tauri` for data consistency across the entire application.               | `ApplicationState/*`                     |
| **Secure & Performant IPC**     | Use `gRPC` for all communication with the `Cocoon`&#x2001;🦋 sidecar, ensuring a well-defined and high-performance API boundary.              | `Vine/*`                                 |
| **UI-Backend Decoupling**       | Interact with `Wind`&#x2001;🍃 exclusively through asynchronous `Tauri` commands and events, keeping the backend UI-agnostic.                 | `Binary/*` (invoke handler), `Command/*` |

---

## System Architecture

```mermaid
graph LR
    classDef mountain fill:#f0d0ff,stroke:#9b59b6,stroke-width:2px,color:#2c0050;
    classDef cocoon   fill:#d0d8ff,stroke:#4a6fa5,stroke-width:2px,color:#001050;
    classDef wind     fill:#cce8ff,stroke:#2980b9,stroke-width:2px,color:#00304a;
    classDef common   fill:#d4f5d4,stroke:#27ae60,stroke-width:1px,stroke-dasharray:5 5,color:#0a3a0a;
    classDef ipc      fill:#fff3c0,stroke:#f39c12,stroke-width:1px,stroke-dasharray:5 5,color:#5a3e00;
    classDef air      fill:#e0f4ff,stroke:#2471a3,stroke-width:1px,stroke-dasharray:5 5,color:#001a30;

    subgraph MOUNTAIN["Mountain ⛰️ - Native Rust/Tauri Backend"]
        direction TB
        subgraph INIT["Binary/ - App Lifecycle"]
            TauriRuntime["Tauri Window & WebView 🚀"]:::mountain
            AppState["ApplicationState 🗄️"]:::mountain
        end
        subgraph DISPATCH["Track/ - Request Dispatcher"]
            TrackDispatcher["Track Dispatcher 🔀"]:::mountain
            EchoScheduler["Echo Work-Stealing Scheduler ⚡"]:::mountain
        end
        subgraph RUNTIME["RunTime/ - Effect Engine"]
            AppRunTime["ApplicationRunTime ⚙️"]:::mountain
            EnvProviders["Environment/ Providers\n(FS · Terminal · SCM · UI · Storage…)"]:::mountain
        end
        subgraph IPC_LAYER["IPC/ - Tauri IPC Server"]
            WindHandlers["WindServiceHandlers mod.rs"]:::ipc
        end
        subgraph VINE_LAYER["Vine/ - gRPC Layer"]
            VineServer["Vine gRPC Server (tonic) 🌿"]:::ipc
        end
        subgraph RPC_LAYER["RPC/ - gRPC Handlers"]
            CocoonRPC["CocoonService handlers"]:::mountain
        end
        CommonCrate["Common 🧑🏻‍🏭 - Traits & DTOs 📐"]:::common

        TauriRuntime --> WindHandlers
        WindHandlers --> TrackDispatcher
        TrackDispatcher --> AppRunTime
        AppRunTime --> EnvProviders
        EnvProviders -.implements.-> CommonCrate
        AppState --- AppRunTime
        VineServer --> CocoonRPC
        CocoonRPC --> TrackDispatcher
        EchoScheduler --- AppRunTime
    end

    subgraph CLIENTS["Clients"]
        SkyWind["Sky / Wind - UI WebView 🍃🌌"]:::wind
        CocoonHost["Cocoon - Node.js Extension Host 🦋"]:::cocoon
    end

    TauriRuntime -- hosts --> SkyWind
    SkyWind -- tauri::invoke --> WindHandlers
    WindHandlers -- sky:// events --> SkyWind
    VineServer <-- gRPC :50052 --> CocoonHost
```

**Connection paths:**

| Path                                                    | Protocol                                  | Use Case                                      |
| ------------------------------------------------------- | ----------------------------------------- | --------------------------------------------- |
| `Wind`&#x2001;🍃/`Sky`&#x2001;🌌 → `Mountain`&#x2001;⛰️ | `tauri::invoke` (IPC)                     | UI command dispatch and event push            |
| `Cocoon`&#x2001;🦋 → `Mountain`&#x2001;⛰️               | `gRPC` over TCP on port 50052             | Extension host ↔ native service calls         |
| `Mountain`&#x2001;⛰️ → `Cocoon`&#x2001;🦋               | `gRPC` notifications via `Vine`&#x2001;🌿 | State updates, progress, output channels      |
| `Mountain`&#x2001;⛰️ → `Wind`&#x2001;🍃                 | `sky://` Tauri events                     | Real-time UI state synchronization            |
| `Mountain`&#x2001;⛰️ → `Air`&#x2001;🪁                  | `AirClient` gRPC                          | OTA update checks and file downloads          |
| Environment providers → `Common`&#x2001;🧑🏻‍🏭              | `trait` implementation                    | Abstract service contracts fulfilled natively |

---

## Key Components

| Component           | Path                          | Description                                                                                                                    |
| ------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Library Entry Point | `Source/Library.rs`           | Library entry point, Tauri setup.                                                                                              |
| LandFixTier         | `Source/LandFixTier.rs`       | Compile-time tier variable boot banner.                                                                                        |
| ApplicationState    | `Source/ApplicationState/`    | Thread-safe state machine with DTOs, persistence, recovery, and feature state.                                                 |
| Binary              | `Source/Binary/`              | Application lifecycle, Tauri command registration, initialization, shutdown.                                                   |
| Command             | `Source/Command/`             | Tauri command handlers grouped by domain (Keybinding, LanguageFeature, SCM, TreeView, Hover).                                  |
| Environment         | `Source/Environment/`         | Concrete implementations of Common provider traits (filesystem, documents, terminal, webview, etc.).                           |
| ExtensionManagement | `Source/ExtensionManagement/` | Extension discovery, manifest parsing, NLS resolution, and VSIX installation.                                                  |
| FileSystem          | `Source/FileSystem/`          | Native file-explorer tree-view provider for the workspace sidebar.                                                             |
| IPC                 | `Source/IPC/`                 | Inter-process communication: Tauri IPC server, Wind service handlers, dev logging, Sky event emission.                         |
| ProcessManagement   | `Source/ProcessManagement/`   | Sidecar process lifecycle, Node.js binary resolution (nvm, fnm, asdf, volta, homebrew, shipped).                               |
| RPC                 | `Source/RPC/`                 | gRPC service implementations (CocoonService, CommandService, TelemetryService, WorkspaceService).                              |
| RunTime             | `Source/RunTime/`             | Effect execution engine (ApplicationRunTime, Execute with retry/timeout, graceful Shutdown).                                   |
| Shim                | `Source/Shim/`                | Deep-hook interception layer for VS Code engine events, gated behind `TierShim` (default `None`, zero overhead when disabled). |
| Telemetry           | `Source/Telemetry/`           | Feature flags, runtime gates, metrics recording, and distributed tracing (OpenTelemetry).                                      |
| Track               | `Source/Track/`               | Central request dispatcher routing frontend, sidecar, and webview commands into ActionEffects.                                 |
| Update              | `Source/Update/`              | Application update service: check, download, and apply updates via Air daemon.                                                 |
| Vine                | `Source/Vine/`                | gRPC IPC layer: server implementation, client multiplexer, notification publishing.                                            |
| Workspace           | `Source/Workspace/`           | Workspace file (.code-workspace) parsing and multi-root folder resolution.                                                     |
| Cache               | `Source/Cache/`               | Asset memory mapping and path canonicalization cache.                                                                          |
| Error               | `Source/Error/`               | Unified error types: ConfigurationError, CoreError, FileSystemError, IPCError, ProviderError.                                  |
| Proto               | `Proto/Vine.proto`            | The gRPC contract definition file for the Vine protocol.                                                                       |

---

## Project Structure&#x2001;🗺️

```
Element/Mountain/
├── Source/
│   ├── Library.rs                 # Library root (lib + staticlib crate types)
│   ├── LandFixTier.rs             # Compile-time tier variable boot banner
│   ├── Air/                       # Air daemon client: download, index, search, update
│   ├── ApplicationState/          # State machine: DTOs, persistence, recovery, features
│   ├── Binary/                    # App lifecycle, CLI parsing, Tauri build, command registration
│   │   ├── Main/                  # Entry point, IPC command wiring, app lifecycle, tray
│   │   ├── Build/                 # Tauri build configuration, plugins, scheme handlers
│   │   ├── Initialize/            # CLI argument parsing, log level, runtime/state build
│   │   ├── Register/              # Command, IPC server, status reporter, Wind sync registration
│   │   ├── Service/               # Air, Cocoon, Vine service startup
│   │   ├── Shutdown/              # Graceful runtime and scheduler shutdown
│   │   ├── Extension/             # Extension populate and scan path configuration
│   │   ├── IPC/                   # Tauri IPC command implementations (health, config, process…)
│   │   ├── Tray/                  # System tray enable and icon switching
│   │   └── Debug/                 # Trace logging and WebKit server
│   ├── Cache/                     # Asset memory mapping and path canonicalization
│   ├── Command/                   # Tauri command handlers: keybinding, language features, SCM, tree view
│   ├── Environment/               # Provider implementations: FS, terminal, debug, webview, output…
│   ├── Error/                     # Unified error types
│   ├── ExtensionManagement/       # Extension scanner, default configs, NLS, VSIX installer
│   ├── FileSystem/                # File explorer tree-view provider
│   ├── IPC/                       # IPC core: Tauri server, Wind handlers, encryption, permissions
│   │   ├── WindServiceHandlers/   # 300+ handler implementations (FS, terminal, UI, Git, model…)
│   │   ├── Enhanced/              # Connection pooling, message compression, secure channels
│   │   ├── Common/                # Connection status, health, message types, performance metrics
│   │   └── Security/              # Permission management, roles, security events
│   ├── ProcessManagement/         # Cocoon sidecar lifecycle, Node.js binary resolution
│   ├── RPC/                       # gRPC service implementations: Cocoon, Commands, Telemetry, Workspace
│   ├── RunTime/                   # Effect engine: ApplicationRunTime, execute with retry/timeout
│   ├── Shim/                      # Deep-hook interception layer, gated behind TierShim env var
│   ├── Telemetry/                 # Feature flags, runtime gates, metrics, OpenTelemetry tracing
│   ├── Track/                     # Central dispatcher: frontend commands, sidecar requests, webview
│   ├── Update/                    # OTA update service via Air daemon
│   ├── Vine/                      # gRPC layer: server, client, multiplexer, proto-generated code
│   └── Workspace/                 # Workspace file parsing and multi-root resolution
├── Proto/
│   └── Vine.proto                 # gRPC protocol definition
├── Documentation/
│   └── GitHub/
│       └── DeepDive.md            # In-depth component architecture documentation
├── capabilities/                  # Tauri capability declarations
├── icons/                         # Application icons
├── Test/                          # Integration tests
├── build.rs                       # Build script: tier feature validation, proto compilation
├── tauri.conf.json                # Tauri configuration
└── Cargo.toml
```

---

## In the Land Project

**Mountain**&#x2001;⛰️ is the primary consumer of the
`Common`&#x2001;🧑🏻‍🏭 crate and a key component in the
`Land`&#x2001;🏞️ monorepo. It depends on:

| Dependency           | Role                                                 |
| -------------------- | ---------------------------------------------------- |
| **Common**&#x2001;🧑🏻‍🏭 | Abstract traits, DTOs, and the `ActionEffect` system |
| **Echo**&#x2001;📣   | Work-stealing scheduler for task execution           |
| **Mist**&#x2001;🌫️   | Pub/sub message bus for event-driven workflows       |
| **Air**&#x2001;🪁    | Background daemon for OTA updates                    |
| **Vine**&#x2001;🌿   | gRPC protocol definitions and multiplexer            |
| **Cache**&#x2001;📦  | Memory-mapped asset and path caches                  |

**Mountain**&#x2001;⛰️ connects to:

| Component                              | Protocol                                         | Role                                                 |
| -------------------------------------- | ------------------------------------------------ | ---------------------------------------------------- |
| **Wind**&#x2001;🍃 / **Sky**&#x2001;🌌 | `tauri::invoke` + `sky://` events                | UI WebView - command dispatch and state push         |
| **Cocoon**&#x2001;🦋                   | `gRPC` via `Vine`&#x2001;🌿 on port 50052         | Node.js extension host sidecar                       |
| **Air**&#x2001;🪁                      | `AirClient` gRPC                                 | Background daemon for file indexing, search, updates |
| **Grove**&#x2001;🌳                    | `gRPC` via `Vine`&#x2001;🌿                      | Native Rust/WASM extension host (future)             |

---

## Getting Started&#x2001;🚀

`Mountain` is a `Rust` crate and a core component of the **Land** repository. It
is built as part of the monorepo. For detailed build instructions, see
[`Documentation/GitHub/Building.md`](https://github.com/CodeEditorLand/Land/tree/Current/Documentation/GitHub/Building.md).

### Prerequisites

- **Rust** 1.95 or later (edition 2024)
- **Tauri** v2 CLI (`cargo install tauri-cli`)
- Protocol Buffer compiler (`protoc`) - required for `Vine.proto` codegen
- macOS, Linux, or Windows build toolchain

### Build

```bash
cd Element/Mountain
cargo build --release
```

### Build with Features

```bash
# Default features (all subsystems enabled)
cargo build --release

# Minimal build (no extension host, no terminals)
cargo build --release --no-default-features --features "MistNative"

# Debug build with development features
cargo build --features "Development"
```

### Run

```bash
cargo run --release
```

**Key Dependencies:**

| Crate / Package        | Purpose                                           |
| ---------------------- | ------------------------------------------------- |
| `Common`&#x2001;🧑🏻‍🏭     | Local path dependency - abstract traits & DTOs    |
| `Echo`&#x2001;📣       | Local path dependency - work-stealing scheduler   |
| `Air`&#x2001;🪁        | Local path dependency - OTA update daemon client  |
| `Mist`&#x2001;🌫️       | Local path dependency - pub/sub message bus       |
| `Vine`&#x2001;🌿       | Local path dependency - gRPC protocol definitions |
| `Cache`&#x2001;📦      | Local path dependency - memory-mapped caches      |
| `keyring`              | Secure OS keychain access                         |
| `log` & `env_logger`   | Structured logging                                |
| `portable-pty`         | Cross-platform native PTY for integrated terminal |
| `serde` & `serde_json` | Serialization / deserialization                   |
| `tauri`                | `^2.x` - windowing, WebView, command dispatch     |
| `tokio`                | Async runtime                                     |
| `tonic`                | `gRPC` server implementation                      |
| `opentelemetry`        | Distributed tracing                               |
| `posthog-rs`           | Product analytics                                 |

---

## Security&#x2001;🔒

Mountain enforces security at multiple layers:

| Layer                      | Mechanism                                                                                                         |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Process isolation**      | `Cocoon`&#x2001;🦋 runs as a separate OS process - a crash in an extension does not take down the editor          |
| **gRPC boundary**          | All extension host ↔ native communication crosses the `Vine`&#x2001;🌿 `gRPC` protocol with typed messages        |
| **Permission system**      | `IPC/Permission/` - role-based access control (Admin, Developer, User, Standard) with audit logging               |
| **Message encryption**     | `IPC/Encryption/SecureChannel` - encrypted message channels with configurable security policies                   |
| **Secure storage**         | Native OS keychain via `keyring` for secrets, tokens, and credentials                                             |
| **Certificate management** | `Binary/Build/CertificateManager` - TLS certificate generation, renewal, and health monitoring                    |
| **Path security**          | `Environment/Utility/PathSecurity` - validates file paths to prevent directory traversal attacks                  |

---

## Compatibility

Mountain is designed to be compatible with:

| Target               | Integration                                                         |
| -------------------- | ------------------------------------------------------------------- |
| **Wind**&#x2001;🍃   | Serves as the backend via `Tauri` commands and `sky://` events      |
| **Cocoon**&#x2001;🦋 | Communicates via `gRPC` on port 50052 for extension host operations |
| **Sky**&#x2001;🌌    | Hosts the `Sky` WebView as the primary UI surface                   |
| **Air**&#x2001;🪁    | Connects via `AirClient` gRPC for OTA updates and file operations   |
| **Common**&#x2001;🧑🏻‍🏭 | Implements all abstract service `trait`s from the `Common` crate    |
| **Echo**&#x2001;📣   | Integrates with the work-stealing scheduler for task execution      |
| **Vine**&#x2001;🌿   | Uses the `Vine.proto` gRPC protocol for all IPC                     |
| **Mist**&#x2001;🌫️   | Connects to the pub/sub message bus for event-driven workflows      |
| **Grove**&#x2001;🌳  | Supports native Rust/WASM extension hosting via `gRPC`              |

---

## API Reference

- **[Rust API Documentation](https://rust.documentation.mountain.editor.land/)**&#x2001;📖

---

## Related Documentation

- [Architecture Overview](https://Editor.Land/Doc/architecture) - Internal
  module structure
- [Deep Dive](Documentation/GitHub/DeepDive.md) - In-depth technical details
- [Land Documentation](../../Documentation/GitHub/README.md) - Complete
  documentation index
- [Why `Rust`](https://Editor.Land/Doc/why-rust)
- [Why `Tauri`](https://Editor.Land/Doc/why-tauri)
- [`Cocoon`](https://github.com/CodeEditorLand/Cocoon)&#x2001;🦋 - Extension
  host sidecar
- [`Grove`](https://github.com/CodeEditorLand/Grove)&#x2001;🌳 - Native
  Rust/WASM extension host
- [`Vine`](https://github.com/CodeEditorLand/Vine)&#x2001;🌿 - gRPC protocol
- [`Echo`](https://github.com/CodeEditorLand/Echo)&#x2001;📣 - Work-stealing
  scheduler
- [`Air`](https://github.com/CodeEditorLand/Air)&#x2001;🪁 - Background daemon
- [`Mist`](https://github.com/CodeEditorLand/Mist)&#x2001;🌫️ - Pub/sub message
  bus

---

## License&#x2001;⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license. You are free to use, modify, distribute, and build upon
this work for any purpose, without any restrictions. For the full legal text,
see the [`LICENSE`](https://github.com/CodeEditorLand/Mountain/tree/Current/LICENSE)
file.

---

## Changelog&#x2001;📜

Stay updated with our progress! See
[`CHANGELOG.md`](https://github.com/CodeEditorLand/Mountain/tree/Current/CHANGELOG.md)
for a history of changes.

---

## Funding & Acknowledgements&#x2001;🙏🏻

**Land**&#x2001;🏞️ is proud to be an open-source endeavor. Our journey is
significantly supported by the organizations and projects that believe in the
future of open-source software.

This project is funded through
[NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
[Next Generation Internet](https://ngi.eu) program. Learn more at the
[NLnet project page](https://NLnet.NL/project/Land).

<table>
	<thead>
		<tr>
			<th align="left">
				<strong>
					Land
				</strong>
			</th>
			<th align="left">
				<strong>
					PlayForm
				</strong>
			</th>
			<th align="left">
				<strong>
					NLnet
				</strong>
			</th>
			<th align="left">
				<strong>
					NGI0 Commons Fund
				</strong>
			</th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td align="left" valign="middle">
				<a href="https://editor.land">
					<img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://PlayForm.Cloud">
					<img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL">
					<img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL/commonsfund">
					<img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund" />
				</a>
			</td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@editor.land](mailto:Source/Open@editor.land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Mountain) |
[Report an Issue](https://github.com/CodeEditorLand/Mountain/issues) |
[Security Policy](https://github.com/CodeEditorLand/Mountain/security/policy)
