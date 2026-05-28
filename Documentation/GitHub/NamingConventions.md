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
	</tr>
</table>

---

# **Naming Conventions** 📝

This document defines the comprehensive naming conventions used throughout the
**Mountain** codebase and the broader Land ecosystem. These conventions ensure
consistency, maintainability, and alignment with cross-language interoperability
requirements.

---

## Table of Contents

- [Overview](#overview)
- [Rationale](#rationale)
- [Complete Naming Patterns](#complete-naming-patterns)
- [Examples by Category](#examples-by-category)
- [Allowed Exceptions](#allowed-exceptions)
- [Migration Guidelines](#migration-guidelines)
- [Implementation Status](#implementation-status)

---

## Overview

**Mountain** adopts **PascalCase** as the primary naming convention for nearly
all Rust elements, diverging from Rust's traditional `snake_case` conventions.
This intentional design choice serves critical ecosystem requirements:

- **Cross-Language DTO Alignment**: All Data Transfer Objects (DTOs) must map
  directly between Rust, TypeScript, and gRPC Protocol Buffers
- **gRPC/Protocol Buffer Compatibility**: gRPC services and messages use
  PascalCase by specification
- **Ecosystem Consistency**: Other components (Cocoon/Node.js sidecar,
  Wind/TypeScript UI) use PascalCase
- **TypeScript Interoperability**: Seamless translation between Rust types and
  TypeScript interfaces

The following attribute is placed at the top of all modules that follow this
convention:

```rust
#![allow(non_snake_case, non_camel_case_types)]
```

This attribute signals to the Rust compiler and developers that the intentional
deviation from Rust conventions is deliberate and approved.

---

## Rationale

### 1. Ecosystem Consistency

The Land ecosystem consists of multiple language-specific components that must
interoperate seamlessly:

| Component     | Language           | Naming Convention                |
| ------------- | ------------------ | -------------------------------- |
| **Mountain**  | Rust               | **PascalCase** (this convention) |
| **Cocoon**    | Node.js/TypeScript | **PascalCase**                   |
| **Wind**      | TypeScript/React   | **PascalCase**                   |
| **Common**    | Rust (shared)      | **PascalCase**                   |
| **Vine gRPC** | Protocol Buffers   | **PascalCase**                   |

Using PascalCase across all components eliminates cognitive overhead during
cross-language development and ensures type safety in gRPC interfaces.

### 2. DTO Alignment

Data Transfer Objects (DTOs) in Rust map directly to:

- **Protocol Buffer Messages** (gRPC service contracts)
- **TypeScript Interfaces** (Wind UI)
- **JSON Schema** ( serialization/deserialization)

Example of DTO alignment:

```rust
// Rust DTO in Mountain/Common
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkSpaceFolderStateDTO {
    pub URI: url::Url,
    pub Name: String,
    pub Index: usize,
}
```

```protobuf
// Protocol Buffer message
message WorkSpaceFolderStateDTO {
    string URI = 1;
    string Name = 2;
    uint32 Index = 3;
}
```

```typescript
// TypeScript interface
interface WorkSpaceFolderStateDTO {
	URI: string;
	Name: string;
	Index: number;
}
```

### 3. gRPC Compatibility

gRPC services and Protocol Buffer messages **require** PascalCase by
specification. Using PascalCase in Rust eliminates the need for custom field
name mappings and serialization aliases.

### 4. Implementation Benefits

- **Simplified Interop Protocol**: No need for `#[serde(rename = "...")]`
  attributes for DTOs
- **Type Safety**: Direct mapping prevents typos between Rust and TypeScript
- **Developer Experience**: Consistent naming across the entire stack
- **Build Simplicity**: No custom protobuf code generation filters needed

---

## Complete Naming Patterns

| Element Type            | Convention               | Example                       | Notes                              |
| ----------------------- | ------------------------ | ----------------------------- | ---------------------------------- |
| **Structs**             | PascalCase               | `WorkSpaceFolderStateDTO`     | All structs, including DTOs        |
| **Enums**               | PascalCase               | `CommandHandler`              | All enum variants                  |
| **Traits**              | PascalCase               | `ConfigurationProvider`       | All trait names                    |
| **Functions**           | PascalCase               | `GetConfigurationValue`       | All public and private functions   |
| **Methods**             | PascalCase               | `CreateWorkSpace`             | All instance methods               |
| **Modules**             | PascalCase               | `ApplicationState`            | File and module names PascalCase   |
| **Constants**           | PascalCase               | `MAX_CONNECTIONS`             | Global constants                   |
| **Static Items**        | PascalCase               | `SIDECAR_CLIENTS`             | Static variables                   |
| **Type Aliases**        | PascalCase               | `CocoonClient`                | Type aliases and generic types     |
| **Generics**            | PascalCase               | `TCapabilityProvider`         | Type parameters with 'T' prefix    |
| **Lifetimes**           | lowercase with 'a prefix | `'a`, `'result`               | Standard Rust lifetime conventions |
| **Local Variables**     | PascalCase               | `WorkSpaceIdentifier`         | All local variables                |
| **Function Parameters** | PascalCase               | `ApplicationHandle:AppHandle` | All parameters                     |
| **Fields**              | PascalCase               | `ActiveDocuments`             | All struct fields                  |
| **File Names**          | PascalCase               | `ApplicationState.rs`         | Rust source files                  |

---

## Examples by Category

### Structs

```rust
// Data Transfer Objects
pub struct WorkSpaceFolderStateDTO {
    pub URI: url::Url,
    pub Name: String,
    pub Index: usize,
}

pub struct MergedConfigurationStateDTO { /* ... */ }

// Core Environment
pub struct MountainEnvironment {
    pub ApplicationHandle: AppHandle<Wry>,
    pub ApplicationState: Arc<ApplicationState>,
}

// Service Providers
pub struct ApplicationRunTime {
    pub Scheduler: Arc<Scheduler>,
    pub Environment: Arc<MountainEnvironment>,
}

// Application State
pub struct ApplicationState {
    pub WorkSpaceFolders: Arc<StandardMutex<Vec<WorkSpaceFolderStateDTO>>>,
    pub Configuration: Arc<StandardMutex<MergedConfigurationStateDTO>>,
    // ... more fields
}
```

### Enums

```rust
// Command Handler Types
pub enum CommandHandler<R: Runtime + 'static> {
    Native(
        fn(
            AppHandle<R>,
            WebviewWindow<R>,
            Arc<ApplicationRunTime>,
            Value,
        ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
    ),
    Proxied {
        SideCarIdentifier: String,
        CommandIdentifier: String,
    },
}
```

### Traits

```rust
// Service Traits from Common
#[async_trait]
impl ConfigurationProvider for MountainEnvironment {
    async fn GetConfigurationValue(
        &self,
        Section: Option<String>,
        Overrides: ConfigurationOverridesDTO,
    ) -> Result<Value, CommonError> {
        // Implementation
    }
}

// Provider Implementation
#[async_trait]
impl DocumentProvider for MountainEnvironment {
    async fn OpenTextDocument(&self, URI: String) -> Result<TextDocument, CommonError> {
        // Implementation
    }
}
```

### Functions and Methods

```rust
// Public API Functions
pub async fn ExecuteCommand(&self, CommandIdentifier: String, Argument: Value) -> Result<Value, CommonError> {
    // Implementation
}

// Internal Helper Functions
pub fn GetConfigurationValue(&self, Section: Option<String>) -> Result<Value, CommonError> {
    // Implementation
}

// Builder/Factory Methods
pub fn Create(Scheduler: Arc<Scheduler>, Environment: Arc<MountainEnvironment>) -> Self {
    Self { Scheduler, Environment }
}

// Utility Functions
pub fn ParseWorkSpaceFile(
    WorkSpaceFilePath: &Path,
    FileContent: &str,
) -> Result<Vec<WorkSpaceFolderStateDTO>, CommonError> {
    // Implementation
}
```

### Modules and Files

```
Source/
├── ApplicationState/
│   ├── ApplicationUserState.rs
│   ├── mod.rs
│   └── DTO/
│       ├── WorkSpaceFolderStateDTO.rs
│       └── ConfigurationStateDTO.rs
├── Environment/
│   ├── MountainEnvironment.rs
│   ├── CommandProvider.rs
│   └── ConfigurationProvider.rs
├── RunTime/
│   └── ApplicationRunTime.rs
└── IPC/
    ├── TauriIPCServer.rs
    └── WindServiceHandlers.rs
```

### Constants and Statics

```rust
// Global Constants
pub const MAX_CONNECTIONS: usize = 100;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// Static Variables
lazy_static! {
    static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// Type Aliases
type CocoonClient = CocoonServiceClient<Channel>;
```

### Generics and Type Parameters

```rust
// Generic Types with PascalCase
pub fn CreateEffectForRequest<R: Runtime>(
    ApplicationHandle: &AppHandle<R>,
    Command: &str,
    Argument: Value,
) -> Result<EffectFn<R>, String> {
    // Implementation
}

// Type Constraints with PascalCase
async fn Run<TCapabilityProvider, TError, TOutput>(
    &self,
    Effect: ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
) -> Result<TOutput, TError>
where
    TCapabilityProvider: ?Sized + Send + Sync + 'static,
    TError: From<CommonError> + Send + Sync + 'static,
    TOutput: Send + Sync + 'static,
{
    // Implementation
}
```

### Local Variables and Parameters

```rust
async fn ProcessRequest(&self, RequestIdentifier: String) -> Result<Value, CommonError> {
    let WorkSpaceIdentifier = self.GetWorkSpaceIdentifier()?;

    let ConfigurationGuard = self
        .ApplicationState
        .Configuration
        .lock()
        .map_err(MapLockError)?;

    let FilePath = WorkSpaceIdentifier.to_string();

    // Processing logic
    Ok(Value::Null)
}
```

---

## Allowed Exceptions

### 1. External Crate Types

**Exception**: Types from external crates retain their original naming
conventions.

```rust
// tokio::spawn, tokio::time::sleep - retain original crate conventions
tokio::spawn(async move { /* ... */ });
tokio::time::sleep(Duration::from_millis(500)).await;

// tauri::AppHandle, tauri::Manager - retain Tauri's conventions
pub struct MountainEnvironment {
    pub ApplicationHandle: AppHandle<Wry>,
}

// serde_json::Value, serde_json::json - retain serde conventions
let Data: Value = json!({ "Key": "Value" });
```

### 2. Standard Library Types

**Exception**: Standard library types and methods retain Rust conventions.

```rust
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    path::PathBuf,
};

// Standard library types
let Map: HashMap<String, Value> = HashMap::new();
let Path = PathBuf::from("/path/to/file");

// Standard library methods
Path.extension();
File.read_to_string(&mut Buffer)?;
```

### 3. Lifetime Parameters

**Exception**: Lifetime parameters use Rust convention (lowercase with 'a
prefix).

```rust
pub struct BorrowHolder<'a> {
    Data: &'a str,
}

pub fn Compare<'a, 'b>(First: &'a str, Second: &'b str) -> bool {
    First == Second
}
```

### 4. Attributes and Macros

**Exception**: Built-in attributes and macros retain Rust conventions.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct MyStruct { /* ... */ }

#[async_trait]
impl MyTrait for MyStruct { /* ... */ }

#[command]
pub async fn MyCommand(Payload: Value) -> Result<Value, String> {
    // Implementation
}
```

### 5. Raw Identifiers and System Constants

**Exception**: Raw identifiers and system-level constants may use lowercase.

```rust
// Raw identifiers for keywords
pub mod r#async { /* ... */ }

// System-level constants
const PATH_SEPARATOR: char = std::path::MAIN_SEPARATOR;

// Environment variables
std::env::var("PATH")?;
```

---

## Migration Guidelines

### For New Code

**Rule**: All new code in Mountain must follow PascalCase conventions.

**Checklist** when creating new modules:

1. ✅ Add `#![allow(non_snake_case, non_camel_case_types)]` at the top 2. ✅ Use
PascalCase for struct/enum/trait names 3. ✅ Use PascalCase for all functions
and methods 4. ✅ Use PascalCase for all fields and variables 5. ✅ Align DTO
fields with Protocol Buffer message fields 6. ✅ Use PascalCase for generic type
parameters with 'T' prefix 7. ✅ Use PascalCase for file names

**Example**:

```rust
#![allow(non_snake_case, non_camel_case_types)]

//! # MyNewModule
//!
//! Brief description of the module.
//!
//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/tree/Current/Documentation/GitHub/NamingConventions.md

use std::sync::Arc;

pub struct MyNewService {
    pub ApplicationState: Arc<ApplicationState>,
}

impl MyNewService {
    pub fn Create(ApplicationState: Arc<ApplicationState>) -> Self {
        Self { ApplicationState }
    }

    pub async fn ProcessRequest(&self, RequestID: String) -> Result<Value, CommonError> {
        // Implementation
        Ok(Value::Null)
    }
}
```

### For Existing Code

**Gradual Migration**:

1. **High-Priority Files**: Start with files that are frequently modified or
   shared with other components
    - DTO files (all `*DTO.rs` files)
    - IPC and gRPC interfaces
    - Provider implementations

2. **Medium-Priority Files**: Service implementations and state management
    - Environment providers
    - Application state components
    - Runtime components

3. **Low-Priority Files**: Utilities, helpers, and isolated modules

**Migration Steps**:

1. Add the header comment with the naming convention reference
2. Verify all struct/enum/field names use PascalCase
3. Update function/method names to PascalCase
4. Update local variable names to PascalCase
5. Run tests to ensure no breaking changes
6. Update documentation if needed

**Example Migration**:

```rust
// Before (snake_case)
pub struct my_struct {
    pub field_name: String,
}

impl my_struct {
    pub fn new(value: String) -> Self {
        Self { field_name: value }
    }

    pub fn do_something(&self, param: &str) -> Result<(), Error> {
        Ok(())
    }
}

// After (PascalCase)
pub struct MyStruct {
    pub FieldName: String,
}

impl MyStruct {
    pub fn Create(Value: String) -> Self {
        Self { FieldName: Value }
    }

    pub fn DoSomething(&self, Param: &str) -> Result<(), Error> {
        Ok(())
    }
}
```

### Renaming Impact Assessment

Before renaming, assess the impact:

1. **Public API**: Does this rename break the public API?
    - If yes, consider adding a deprecated alias first
    - Document the breaking change in CHANGELOG.md

2. **gRPC Interfaces**: Does this affect Protocol Buffer definitions?
    - Must update `.proto` files and regenerate code
    - Verify compatibility with Cocoon and Wind

3. **Tests**: Update all test references to use new names
    - Unit tests
    - Integration tests
    - E2E tests

4. **Documentation**: Update inline documentation and comments

---

## Implementation Status

### Core Modules ✅

| Module             | Status      | Notes                                                  |
| ------------------ | ----------- | ------------------------------------------------------ |
| `ApplicationState` | ✅ Complete | All structs, DTOs, and methods follow PascalCase       |
| `Environment`      | ✅ Complete | All providers and helpers follow PascalCase            |
| `RunTime`          | ✅ Complete | ApplicationRunTime and all helpers follow PascalCase   |
| `Track`            | ✅ Complete | DispatchLogic and EffectCreation follow PascalCase     |
| `Vine`             | ✅ Complete | Server and gRPC integration follow PascalCase          |
| `IPC`              | ✅ Complete | All IPC handlers and adapters follow PascalCase        |
| `WorkSpace`        | ✅ Complete | WorkspaceProvider follows PascalCase                   |

### DTOs ✅

All DTO files in `ApplicationState/DTO/` follow PascalCase conventions:

- `CustomDocumentStateDTO.rs`
- `DocumentStateDTO.rs`
- `ExtensionDescriptionStateDTO.rs`
- `MarkerDataDTO.rs`
- `MergedConfigurationStateDTO.rs`
- `OutputChannelStateDTO.rs`
- `ProviderRegistrationDTO.rs`
- `RPCModelContentChangeDTO.rs`
- `TerminalStateDTO.rs`
- `TreeViewStateDTO.rs`
- `WebViewStateDTO.rs`
- `WindowStateDTO.rs`
- `WorkSpaceFolderStateDTO.rs`

### Environment Providers ✅

All provider implementations in `Environment/` follow PascalCase:

- `MountainEnvironment.rs`
- `CommandProvider.rs`
- `ConfigurationProvider.rs`
- `CustomEditorProvider.rs`
- `DebugProvider.rs`
- `DiagnosticProvider.rs`
- `DocumentProvider.rs`
- `FileSystemProvider.rs`
- `IPCProvider.rs`
- `KeybindingProvider.rs`
- `LanguageFeatureProvider.rs`
- `OutputProvider.rs`
- `SearchProvider.rs`
- `SecretProvider.rs`
- `SourceControlManagementProvider.rs`
- `StatusBarProvider.rs`
- `StorageProvider.rs`
- `SynchronizationProvider.rs`
- `TerminalProvider.rs`
- `TestProvider.rs`
- `TreeViewProvider.rs`
- `UserInterfaceProvider.rs`
- `WebViewProvider.rs`
- `WorkSpaceProvider.rs`

### Module Headers ✅

All modules include the naming convention header:

```rust
// At the top of each .rs file (after the initial doc comments):
//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/tree/Current/Documentation/GitHub/NamingConventions.md
```

---

## Further Reading

- [Deep Dive & Architecture](https://github.com/CodeEditorLand/Mountain/tree/Current/Documentation/GitHub/DeepDive.md)
- [Common Crate Documentation](https://github.com/CodeEditorLand/Common)
- [gRPC Best Practices](https://grpc.io/docs/guides/)
- [Protocol Buffers Style Guide](https://developers.google.com/protocol-buffers/docs/style)

---

## Quick Reference Card

```
Rust in Mountain/Cocoon/Wind
├── Structs:        PascalCase    WorkSpaceFolderStateDTO
├── Enums:          PascalCase    CommandHandler
├── Traits:         PascalCase    ConfigurationProvider
├── Functions:      PascalCase    GetConfigurationValue
├── Methods:        PascalCase    CreateWorkSpace
├── Modules:        PascalCase    ApplicationState
├── Constants:      PascalCase    MAX_CONNECTIONS
├── TypeAlias:      PascalCase    CocoonClient
├── Generics:       PascalCase    T (single), TCapability (multi)
├── Fields:         PascalCase    ActiveDocuments
├── Variables:      PascalCase    WorkSpaceIdentifier
├── Parameters:     PascalCase    ApplicationHandle
├── File Names:     PascalCase    ApplicationState.rs
└── Attributes:     PascalCase    #[allow(non_snake_case, non_camel_case_types)]
```

---

## Changelog

| Date       | Version | Changes                                  |
| ---------- | ------- | ---------------------------------------- |
| 2026-01-28 | 1.0.0   | Initial naming conventions documentation |

---

**Maintained by**: [CodeEditorLand Team](https://Editor.Land)

**License**: See
[LICENSE](https://github.com/CodeEditorLand/Mountain/tree/Current/LICENSE)
