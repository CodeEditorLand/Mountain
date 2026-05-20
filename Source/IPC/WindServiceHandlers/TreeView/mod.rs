#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! TreeView atoms - renderer-side tree-view RPC handlers.
//!
//! `GetChildren` - `tree:getChildren` IPC → `$provideTreeChildren` Cocoon RPC.

pub mod GetChildren;
