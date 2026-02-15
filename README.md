<table>
	<tr>
		<td align="left" valign="middle">
			<h3 align="left"> Mountain</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				⛰️
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left"> + </h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://Editor.Land" target="_blank">
					<picture>
						<source media="(prefers-color-scheme: dark)" srcset="https://PlayForm.Cloud/Dark/Image/GitHub/Land.svg">
						<source media="(prefers-color-scheme: light)" srcset="https://PlayForm.Cloud/Image/GitHub/Land.svg">
						<img width="28" alt="Land Logo" src="https://PlayForm.Cloud/Image/GitHub/Land.svg">
					</picture>
				</a>
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://Editor.Land" target="_blank">
					Land
				</a>
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				🏞️
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left"> + </h3>
		</td>
		<td align="left" valign="middle" width="190">
			<h3 align="left">
				<a href="https://Tauri.App" target="_blank">
					<picture>
						<source media="(prefers-color-scheme: dark)" srcset="https://PlayForm.Cloud/Dark/Image/GitHub/Made/Tauri.svg">
						<source media="(prefers-color-scheme: light)" srcset="https://PlayForm.Cloud/Image/GitHub/Made/Tauri.svg">
						<img width="160" alt="Made With Tauri" src="https://PlayForm.Cloud/Image/GitHub/Made/Tauri.svg">
					</picture>
				</a>
			</h3>
		</td>
	</tr>
</table>

---

# **Mountain** ⛰️ The Bedrock of Land: Native Backend & Service Host

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Mountain/tree/Current/LICENSE)
[![Rust Version](https://img.shields.io/badge/Rust-1.77+-blue.svg)](https://www.rust-lang.org/)
[![Tauri Version](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![Tonic gRPC Version](https://img.shields.io/badge/Tonic-v0.11-blueviolet.svg)](https://github.com/hyperium/tonic)

Welcome to **Mountain**! This element is the native Rust backend and Tauri
application shell for the Land Code Editor. It serves as the foundational
bedrock for the entire system, managing the application lifecycle, orchestrating
native OS operations, and providing high-performance services to the `Wind`
frontend and the `Cocoon` extension host.

**Mountain** is engineered to:

1.  **Be the Native Core:** Act as the primary Rust application, leveraging
    Tauri to create a lightweight, cross-platform windowing and webview host.
2.  **Provide High-Performance Services:** Implement the abstract service traits
    defined in the `Common` crate, offering native-speed implementations for
    filesystem I/O, process management, secure storage, and more.
3.  **Orchestrate Sidecars:** Reliably launch, manage, and communicate with the
    `Cocoon` (Node.js) extension host sidecar via a robust gRPC interface.
4.  **Power the User Interface:** Serve as the backend for the `Wind` User
    Interface layer, responding to requests via Tauri commands and pushing state
    updates via Tauri events.

---

## Key Features 🔐

- **Declarative Effect System:** Built on a Rust `ActionEffect` system defined
  in the `Common` crate. Business logic is described as declarative, composable
  effects, which are executed by a central `ApplicationRunTime`.
- **gRPC-Powered IPC:** Hosts a `tonic`-based gRPC server (`Vine`) to provide a
  strongly-typed, high-performance communication channel for the `Cocoon`
  extension host.
- **Centralized State Management:** Utilizes a thread-safe, Tauri-managed
  `ApplicationState` to act as the single source of truth for the entire
  application's state, from open documents to provider registrations.
- **Native PTY Management:** Implements a full-featured integrated terminal
  service by spawning and managing native pseudo-terminals (`PTY`) using the
  `portable-pty` crate.
- **Secure Storage Integration:** Leverages the native OS keychain via the
  `keyring` crate to securely store sensitive data like authentication tokens.
- **Robust Command Dispatching:** A central `Track` dispatcher intelligently
  routes all incoming requests from the User Interface (`Wind`) and extensions
  (`Cocoon`) to the appropriate native `Environment` provider or `ActionEffect`.

---

## Core Architecture Principles 🏗️

| Principle                             | Description                                                                                                                                        | Key Components Involved                          |
| :------------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------- |
| **Implementation of Contracts**       | Faithfully implement the abstract service `trait`s defined in the `Common` crate, providing the concrete logic for the application's architecture. | `Environment/*` providers                        |
| **Separation of Concerns**            | Isolate service logic into distinct `Environment` provider modules, each responsible for a specific domain (e.g., FileSystem, Documents).          | `Environment/*`, `Command/*`                     |
| **Declarative Logic**                 | Express complex operations as `ActionEffect`s, which are executed by the `ApplicationRunTime`. This makes logic composable, testable, and robust.  | `RunTime/*`, `Track/EffectCreation.rs`, `Common` |
| **Centralized State**                 | Maintain a single, thread-safe `ApplicationState` struct managed by Tauri to ensure data consistency across the entire application.                | `ApplicationState/*`                             |
| **Secure & Performant IPC**           | Utilize gRPC for all communication with the `Cocoon` sidecar, ensuring a well-defined and high-performance API boundary.                           | `Vine/*`                                         |
| **User Interface-Backend Decoupling** | Interact with the `Wind` frontend exclusively through asynchronous Tauri commands and events, ensuring the backend is User Interface-agnostic.     | `Binary.rs` (invoke handler), `Command/*`        |

---

## Deep Dive & Component Breakdown 🔬

To understand how `Mountain`'s internal components are structured and how they
implement the application's core logic, please refer to the detailed technical
breakdown in [`Documentation/GitHub/DeepDive.md`](https://github.com/CodeEditorLand/Mountain/tree/Current/Documentation/GitHub/DeepDive.md). This document explains
the roles of the `ApplicationRunTime`, `ApplicationState`, `Handler`,
`Environment`, and the `Vine` gRPC layer.

---

## `Mountain` in the Land Ecosystem ⛰️ + 🏞️

This diagram illustrates `Mountain`'s central role as the native orchestrator
for the entire Land application.

```mermaid
graph LR
    classDef Mountain fill:#f9f,stroke:#333,stroke-width:2px;
    classDef Cocoon fill:#ccf,stroke:#333,stroke-width:2px;
    classDef Wind fill:#9cf,stroke:#333,stroke-width:2px;
    classDef Common fill:#cfc,stroke:#333,stroke-width:1px;
    classDef IPC fill:#ff9,stroke:#333,stroke-width:1px,stroke-dasharray: 5 5;

    subgraph "Mountain (Native Rust/Tauri Backend)"
        TauriRuntime[Tauri App & Window]:::Mountain
        ApplicationRunTime[ApplicationRunTime Engine]:::Mountain
        ApplicationState["ApplicationState (Shared State)"]:::Mountain
        TrackDispatcher[Track Dispatcher]:::Mountain
        VinegRPC[Vine gRPC Server]:::IPC
        EnvironmentProviders[Environment Providers]:::Mountain
        CommonCrate["Common Crate (Traits & DTOs)"]:::Common

        TauriRuntime -- Manages --> ApplicationState
        TauriRuntime -- Manages --> ApplicationRunTime
        ApplicationRunTime -- Executes effects via --> EnvironmentProviders
        TrackDispatcher -- Routes requests to --> ApplicationRunTime
    end

    subgraph "Clients"
        WindUI["Wind/Sky User Interface (Webview)"]:::Wind
        CocoonSideCar["Cocoon Extension Host (Node.js)"]:::Cocoon
    end

    TauriRuntime -- Hosts --> WindUI
    WindUI -- Tauri Command --> TrackDispatcher
    TrackDispatcher -- Tauri Events --> WindUI

    VinegRPC -- gRPC Protocol <--> CocoonSideCar; class VinegRPC,CocoonSideCar IPC
    VinegRPC -- Forwards requests to --> TrackDispatcher

    EnvironmentProviders -- Implements traits from --> CommonCrate
```

---

## Project Structure Overview 🗺️

The `Mountain` repository is organized to clearly separate concerns, following
the architectural patterns defined in `Common`.

```
Mountain/
├── Source/
│   ├── Binary.rs                    # Tauri application entry point and setup.
│   ├── ApplicationState/            # Central, thread-safe state store and its DTOs.
│   ├── Command/                     # Tauri command handlers for UI-specific requests.
│   ├── Environment/                 # Concrete implementations of the `Common` provider traits.
│   ├── ExtensionManagement/         # Logic for scanning and parsing extensions.
│   ├── FileSystem/                  # Native TreeView provider for the File Explorer.
│   ├── ProcessManagement/           # Logic for managing the `Cocoon` sidecar process.
│   ├── RunTime/                     # The `ApplicationRunTime` engine that executes effects.
│   ├── Track/                       # The central request dispatcher (`EffectCreation`).
│   ├── Update/                      # Application self-updating logic.
│   ├── Vine/                        # The gRPC server and client implementation (`tonic`).
│   └── Workspace/                   # Logic for handling `.code-workspace` files.
├── Proto/
│   └── Vine.proto                   # The gRPC contract definition file.
└── build.rs                         # Build script to compile the .proto file into Rust code.
```

---

## Development Setup 🛠️

`Mountain` is a Rust crate and a core component of the main `Land` repository.
It is not intended to be built or run standalone. Please follow the instructions
in the main [Land Repository README](https://github.com/CodeEditorLand/Land) to
set up, build, and run the entire application.

**Key Dependencies:**

- `Common` (local path dependency).
- `Echo` (local path dependency).
- `keyring`: For secure secret storage.
- `log` & `env_logger`: For logging.
- `portable-pty`: For the integrated terminal feature.
- `serde` & `serde_json`: For serialization.
- `tauri`: `^2.x`
- `tokio`: For the asynchronous RunTime.
- `tonic`: For the gRPC server implementation.

---

## License ⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license. You are free to use, modify, distribute, and build upon
this work for any purpose, without any restrictions. For the full legal text,
see the [`LICENSE`](https://github.com/CodeEditorLand/Mountain/tree/Current/) file.

---

## Changelog 📜

Stay updated with our progress! See [`CHANGELOG.md`](https://github.com/CodeEditorLand/Mountain/tree/Current/) for a history
of changes specific to **Mountain**.

---

## Funding & Acknowledgements 🙏🏻

**Mountain** is a core element of the **Land** ecosystem. This project is funded
through [NGI0 Commons Fund](https://NLnet.NL/Commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
[Next Generation Internet](https://ngi.eu) program. Learn more at the
[NLnet project page](https://NLnet.NL/project/Land).

<table>
	<thead>
		<tr>
			<th align="left"><strong>Land</strong></th>
			<th align="left"><strong>PlayForm</strong></th>
			<th align="left"><strong>NLnet</strong></th>
			<th align="left"><strong>NGI0 Commons Fund</strong></th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td align="left" valign="middle">
				<a href="https://Editor.Land">
					<img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://PlayForm.Cloud">
					<img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL">
					<img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL/commonsfund">
					<img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund">
				</a>
			</td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@Editor.Land](mailto:Source/Open@Editor.Land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Mountain) |
[Report an Issue](https://github.com/CodeEditorLand/Mountain/issues) |
[Security Policy](https://github.com/CodeEditorLand/Mountain/security/policy)
