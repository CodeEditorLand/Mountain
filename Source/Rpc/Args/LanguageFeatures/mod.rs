// File: Rpc/Args/LanguageFeatures/Mod.rs
// This module defines the argument structures (DTOs) used for RPC calls
// related to language-specific features like completion, hover, definition,
// etc.

mod EmitProviderEventArgument; // Renamed from Emitprovidereventargument
mod RegisterProviderArgument; // Renamed from Registerproviderargument
mod UnregisterProviderArgument; // Renamed from Unregisterproviderargument

pub use EmitProviderEventArgument::EmitProviderEventArgument;
pub use RegisterProviderArgument::RegisterProviderArgument;
pub use UnregisterProviderArgument::UnregisterProviderArgument;
