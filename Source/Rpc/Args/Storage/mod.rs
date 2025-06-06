// File: Rpc/Args/Storage/Mod.rs
// This module defines the argument structures (DTOs) used for
// RPC calls related to Memento storage (global and workspace).

mod GetValueArgument; // Renamed from Getvalueargument
mod SetValueArgument; // Renamed from Setvalueargument
// TargetDto was part of GetValueArgument and is implicitly handled there.

pub use GetValueArgument::{GetValueArgument, TargetDto}; /* Re-export TargetDto as it's used by
                                                           * SetValueArgument too */
pub use SetValueArgument::SetValueArgument;
