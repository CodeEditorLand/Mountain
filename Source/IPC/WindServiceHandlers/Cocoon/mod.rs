//! Cocoon bridge atoms - renderer→Cocoon gRPC forwarding.
//!
//! `Request`  - two-way RPC (awaits a reply from Cocoon).
//! `Notify`   - fire-and-forget notification (no reply expected).
//! `ExtensionHostMessage` - binary extension-host protocol relay.

pub mod ExtensionHostMessage;

pub mod Notify;

pub mod Request;
