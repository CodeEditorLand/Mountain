#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Generic-request atom modules for `process_mountain_request`.
//!
//! Each submodule handles one semantic group of method names from
//! Cocoon's `MountainGRPCClient.sendRequest(method, params)` rail.
//! Handler functions take `(RequestId, Params, &Env)` and return a
//! typed `Response<GenericResponse>` without referencing `CocoonServiceImpl`.
//!
//! Groups:
//! - `Commands`    - `commands.execute`, `executeCommand`, `unregisterCommand`
//! - `FileSystem`  - `fs.*`, `readFile`, `writeFile`, `stat`, `readdir`, …
//!   Also exports `OkResponse` / `ErrResponse` helpers used by other groups.
//! - `Secrets`     - `getSecret`, `storeSecret`, `deleteSecret`
//! - `WindowDialogs` - dialogs, messages, status bar, webview, workspace ops

pub mod Commands;

pub mod FileSystem;

pub mod Secrets;

pub mod WindowDialogs;
