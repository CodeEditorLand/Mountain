
// This module is responsible for tracking and dispatching commands and requests
// that originate from the Sky frontend or from the Cocoon sidecar.
// It acts as a central routing point, determining how to handle incoming
// messages, whether by executing local effects, invoking RPC handlers, or
// forwarding to other systems.

mod EffectCreationError; // Defines errors related to effect creation
mod SkyCommandDto; // DTOs for commands originating from Sky (frontend)
mod Track; // Contains the main dispatch logic

pub use self::SkyCommandDto::*; // Re-export all DTOs for Sky commands
pub use self::{EffectCreationError::EffectCreationError, Track::*}; // Re-export main dispatch functions
