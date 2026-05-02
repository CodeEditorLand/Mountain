#![allow(non_snake_case)]

//! `MarshalledId.Uri` constant from VS Code's
//! `src/vs/base/common/marshallingIds.ts`. The renderer's URI reviver
//! (`_transformIncomingURIs` in `uriIpc.ts`) keys off this exact value
//! to decide whether to call `URI.revive()`. If VS Code ever renumbers
//! the enum, this constant must move in lockstep.

pub const VALUE:u64 = 1;
