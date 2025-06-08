<table><tr>
<td colspan="1"> <h3 align="center"> <picture>
<source media="(prefers-color-scheme: dark)" srcset="https://PlayForm.Cloud/Dark/Image/GitHub/Land.svg">
<source media="(prefers-color-scheme: light)" srcset="https://PlayForm.Cloud/Image/GitHub/Land.svg">
<img width="28" alt="Land Logo" src="https://PlayForm.Cloud/Image/GitHub/Land.svg">
</picture> </h3> </td> <td colspan="3" valign="top"> <h3 align="center"> Mountain ⛰️
</h3> </td>
</tr></table>

---

# **Mountain** ⛰️ Deep Dive & Architecture

This document provides a detailed technical overview of the **Mountain** project
for developers. It explores the internal architecture, the flow of control from
request to execution, and the design patterns used to create a robust,
effects-based native backend for the Land Code Editor.

---

## Core Architecture Principles

| Principle                       | Description                                                                                                                                        | Key Components Involved                             |
| :------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------- |
| **Implementation of Contracts** | Faithfully implement the abstract service `trait`s defined in the `Common` crate, providing the concrete logic for the application's architecture. | `environment/*` providers                           |
| **Separation of Concerns**      | Isolate business logic in `handlers` modules, keeping the `environment` provider implementations clean and focused on delegation.                  | `environment/*`, `handlers/*`                       |
| **Declarative Logic**           | Express all operations as `ActionEffect`s, which are executed by the `AppRuntime`. This makes logic composable, testable, and robust.              | `runtime/*`, `track/*`, `Common::effect`            |
| **Centralized State**           | Maintain a single, thread-safe `AppState` struct managed by Tauri to ensure data consistency across the entire application.                        | `app_state/*`                                       |
| **Secure & Performant IPC**     | Utilize gRPC for all communication with the `Cocoon` sidecar, ensuring a well-defined and high-performance API boundary.                           | `vine/*`                                            |
| **UI-Backend Decoupling**       | Interact with the `Wind` frontend exclusively through asynchronous Tauri commands and events, ensuring the backend is UI-agnostic.                 | `main.rs` (invoke handler), `handlers/*` (emitters) |

---

## Deep Dive into `Mountain`'s Components

### 1. The `main.rs` Entry Point and Tauri Setup

- **Role:** This is the application's bootstrap sequence.
- **Functionality:**
    - Initializes logging (`env_logger`).
    - Creates the `tauri::Builder` and configures the application.
    - Uses the `.setup()` hook as the primary initialization point. Inside this
      hook, it performs the critical task of creating the singleton instances of
      `AppState`, `MountainEnvironment`, and `AppRuntime` and placing them into
      Tauri's managed state (`AppHandle.manage(...)`).
    - Spawns a background `tokio` task for long-running initializations (like
      scanning for extensions and starting the gRPC server) to avoid blocking
      the main thread and allow the UI to appear faster.
    - Registers all Tauri command handlers (e.g., `track::DispatchCommand`),
      which are the entry points for requests coming from the `Wind` UI.
    - Handles application lifecycle events, such as `RunEvent::ExitRequested`.

### 2. The `runtime` and `environment` Modules (The Execution Core)

- **Role:** These two modules work together to form the execution engine for the
  application's logic.
- **`runtime/`:**
    - `AppRuntime.rs` provides the concrete `AppRuntime` for `Mountain`. It
      holds an instance of the `MountainEnvironment`.
    - Its primary method, `Run`, takes an `ActionEffect` from the `Common`
      crate. It determines the required capability (e.g., `dyn FsReader`), gets
      it from its environment, and then applies the effect, executing the
      wrapped async function.
- **`environment/`:**
    - `MountainEnvironment.rs` is the central struct that **implements all
      provider `trait`s** from `Common`. For example, it contains
      `impl FsReader for MountainEnvironment`.
    - **Crucially, the `impl` blocks in this module contain no business logic.**
      They are a clean "wiring" layer. Each method is a one-line call that
      delegates to a corresponding function in the `handlers` module (e.g.,
      `self.ReadFile(...)` calls `handlers::fs::ReadFileLogic(...)`).
    - `Utils.rs` contains shared helper functions used across multiple provider
      implementations, such as error mapping and security checks.

### 3. The `app_state` Module (The Single Source of Truth)

- **Role:** To provide a single, globally accessible, thread-safe container for
  all of the application's runtime state.
- **Structure:**
    - The `AppState` struct contains fields like `WorkspaceFolders`,
      `Configuration`, `ActiveTerminals`, and `LanguageProviders`.
    - Every field is wrapped in `Arc<Mutex<...>>` (or `Arc<Atomic...>` for
      simple types) to allow for safe, shared access from any asynchronous task
      or thread in the application (e.g., a gRPC request handler and a Tauri
      command handler can both safely access `AppState`).
    - The `default()` implementation initializes the state, reads initial data
      from disk (like Memento storage), and sets up default values.

### 4. The `handlers` Module (The Business Logic Layer)

- **Role:** This is where the actual work gets done. By isolating logic here,
  the `environment` module remains clean, and the business logic is easy to
  find, read, and test.
- **Functionality:**
    - Each submodule (`handlers/fs`, `handlers/terminal`, etc.) corresponds to a
      service domain.
    - Functions within these modules (e.g., `ReadFileLogic`,
      `CreateTerminalLogic`) perform the concrete operations. They take the
      `AppHandle` as an argument, which allows them to access `AppState`.
    - They interact with the native OS using crates like `tokio::fs`,
      `portable-pty`, and `keyring`.
    - After performing an operation and updating `AppState`, they are often
      responsible for emitting Tauri events to notify the `Wind` UI of the
      change (e.g., `AppHandle.emit("sky://terminal/data", ...)`).

### 5. The `vine` and `track` Modules (The Communication & Dispatch Layer)

- **Role:** These modules form the application's "front door" for all external
  requests coming from `Cocoon` and `Wind`.
- **`vine/` (gRPC):**
    - `proto/vine.proto`: Defines the gRPC contract. It is the single source of
      truth for the `Mountain <-> Cocoon` API.
    - `build.rs`: Compiles the `.proto` file into Rust code using `tonic-build`.
    - `server/MountainVineGrpcService.rs`: Implements the `MountainService`
      trait generated by `tonic`. This is the gRPC request handler. When it
      receives a call (e.g., `ProcessCocoonRequest`), its only job is to pass
      the method name and parameters to the `track` dispatcher.
- **`track/` (Dispatcher):**
    - `TrackLogic.rs` is the central router for the entire application. It
      exposes two primary functions: `DispatchCommand` (for Tauri invokes from
      `Wind`) and `DispatchSidecarRequest` (for gRPC calls from `Cocoon`).
    - **Dispatch Strategy:** When a request comes in, the dispatcher first
      consults `EffectCreation.rs`. It attempts to map the request's method name
      and parameters into a declarative `ActionEffect`. If a mapping exists, it
      passes the `ActionEffect` to the `AppRuntime` for execution.
    - **Fallback:** If no `ActionEffect` mapping is found (for legacy or highly
      specific RPCs), it falls back to a direct RPC handler system (not fully
      detailed in the synthesis, but this is the architectural slot for it).

---

## End-to-End Workflow Example: `CreateTerminal`

This demonstrates how all the components work together in a typical flow.

1.  **Request Origin:** `Cocoon` sends a `CreateTerminal` gRPC request to
    `Mountain`.
2.  **Vine (gRPC Server):** `MountainVineGrpcService` receives the gRPC call. It
    extracts the method name (`"$createTerminal"`) and parameters. It passes
    these to `track::DispatchSidecarRequest`.
3.  **Track (Dispatcher):** The dispatcher looks up `"$createTerminal"` in
    `EffectCreation`. It finds a match and constructs a
    `Common::terminal::CreateTerminal` `ActionEffect`.
4.  **Runtime:** The dispatcher calls `AppRuntime.Run(effect)`.
5.  **Environment (Execution):**
    - The `AppRuntime` sees that the effect requires the `dyn TerminalProvider`
      capability.
    - It gets the `MountainEnvironment` and calls its `.Require()` method to get
      an `Arc<dyn TerminalProvider>`.
    - It applies the effect, which invokes the
      `TerminalProvider::CreateTerminal` method on the `MountainEnvironment`.
6.  **Provider (Delegation):** The
    `impl TerminalProvider for MountainEnvironment` block immediately delegates
    the call to `handlers::terminal::CreateTerminalLogic`, passing along its
    `AppHandle`.
7.  **Handler (Business Logic):**
    - The `CreateTerminalLogic` function performs the actual work.
    - It gets a new ID from `AppState`.
    - It uses `portable-pty` to spawn a native shell process.
    - It creates a `TerminalStateDto` and stores it in `AppState`.
    - It spawns `tokio` tasks to handle I/O streaming.
    - It sends gRPC notifications (`$acceptTerminalOpened`) back to `Cocoon`.
    - It emits Tauri events (`sky://terminal/create`) to `Wind`.
    - It returns a `Result` indicating success or failure.
8.  **Unwinding:** The `Result` unwinds back up the call stack: from the
    handler, through the environment, out of the `AppRuntime`, through the
    dispatcher, and finally, `MountainVineGrpcService` sends it back to `Cocoon`
    as the gRPC response.
