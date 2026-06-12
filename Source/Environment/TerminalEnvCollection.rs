//! Global registry of per-extension `EnvironmentVariableCollection`
//! mutations.
//!
//! Each extension that calls `context.environmentVariableCollection.replace(
//! variable, value)` lands a `Mutator` in this registry keyed by
//! `(ExtensionId, VariableName)`. Every PTY spawn consults the registry
//! and applies the accumulated mutations to the child env BEFORE the
//! shell process is launched, so terminals created after an extension's
//! activation observe the env it requested.
//!
//! Persistence: `persistent=true` mutations survive a window reload via
//! Mountain's storage provider (key
//! `__envCollection:<extensionId>`). Non-persistent mutations live only
//! for the current session.
//!
//! Wire format matches VS Code's `EnvironmentVariableMutator`:
//!   • Type::Replace (1) - set to `value`, ignore inherited
//!   • Type::Append  (2) - `inherited + value`
//!   • Type::Prepend (3) - `value + inherited`
//!
//! `applyAtProcessCreation` (default true) is the only application
//! point in the Tauri+PTY world; the upstream `applyAtShellIntegration`
//! second-chance hook is irrelevant when we already control the spawn.

use std::{
	collections::HashMap,
	sync::{Mutex, OnceLock},
};

use serde_json::Value;

/// Supported environment variable mutation operations.
///
/// Mirrors VS Code's `EnvironmentVariableMutatorType`:
/// `Replace` sets the value outright, `Append` adds to the inherited value,
/// `Prepend` prefixes the inherited value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MutatorType {
	/// Replaces the variable value entirely, discarding the inherited value.
	Replace = 1,

	/// Appends the value to the inherited variable value.
	Append = 2,

	/// Prepends the value before the inherited variable value.
	Prepend = 3,
}

/// A single environment variable mutation registered by an extension.
///
/// Carries the variable name, the value to apply, and the mutation strategy.
#[derive(Clone, Debug)]
pub struct Mutator {
	/// The name of the environment variable to mutate.
	pub Variable:String,

	/// The value to set, append, or prepend.
	pub Value:String,

	/// The mutation strategy (Replace, Append, or Prepend).
	pub Kind:MutatorType,
}

/// All environment variable mutations registered by a single extension.
///
/// Each extension gets one `ExtensionCollection` holding its mutations along
/// with persistence and description metadata.
#[derive(Clone, Default)]
pub struct ExtensionCollection {
	/// Whether these mutations survive a window reload.
	pub Persistent:bool,

	/// User-facing label for this collection.
	pub Description:Option<String>,

	/// Per-variable mutations registered by this extension.
	pub Mutators:HashMap<String, Mutator>,
}

static REGISTRY:OnceLock<Mutex<HashMap<String, ExtensionCollection>>> = OnceLock::new();

fn Get() -> &'static Mutex<HashMap<String, ExtensionCollection>> { REGISTRY.get_or_init(|| Mutex::new(HashMap::new())) }

/// Replaces the value of a variable for the given extension, discarding any
/// inherited value.
pub fn Replace(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Replace });
	}
}

/// Appends `value` to the inherited value of `variable` for the given
/// extension.
pub fn Append(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Append });
	}
}

/// Prepends `value` before the inherited value of `variable` for the given
/// extension.
pub fn Prepend(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Prepend });
	}
}

/// Removes the variable mutation for `variable` from the given extension's
/// collection. Idempotent: calling for a non-existent variable is a no-op.
pub fn Delete(ExtensionId:&str, Variable:&str) {
	if let Ok(mut Guard) = Get().lock() {
		if let Some(Entry) = Guard.get_mut(ExtensionId) {
			Entry.Mutators.remove(Variable);
		}
	}
}

/// Removes all variable mutations for the given extension.
pub fn Clear(ExtensionId:&str) {
	if let Ok(mut Guard) = Get().lock() {
		if let Some(Entry) = Guard.get_mut(ExtensionId) {
			Entry.Mutators.clear();
		}
	}
}

/// Sets whether the given extension's mutations survive a window reload.
pub fn SetPersistent(ExtensionId:&str, Persistent:bool) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry.Persistent = Persistent;
	}
}

/// Sets a user-facing description for the given extension's collection.
pub fn SetDescription(ExtensionId:&str, Description:Option<String>) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry.Description = Description;
	}
}

/// Apply every registered mutation across every extension to the supplied
/// env map. Mutations are deterministic per (extensionId, variable);
/// across extensions the order is iteration-stable but unordered, so
/// avoid relying on cross-extension ordering for the same variable
/// (matches VS Code's documented behavior).
pub fn ApplyToEnv(Env:&mut HashMap<String, String>) {
	let Snapshot = match Get().lock() {
		Ok(Guard) => Guard.clone(),

		Err(_) => return,
	};

	for (_ExtId, Collection) in Snapshot {
		for Mut in Collection.Mutators.values() {
			let Inherited = Env.get(&Mut.Variable).cloned().unwrap_or_default();

			let Next = match Mut.Kind {
				MutatorType::Replace => Mut.Value.clone(),

				MutatorType::Append => format!("{}{}", Inherited, Mut.Value),

				MutatorType::Prepend => format!("{}{}", Mut.Value, Inherited),
			};

			Env.insert(Mut.Variable.clone(), Next);
		}
	}
}

/// Parse the wire payload `{ extensionId, variable, value, persistent,
/// description }` into a uniform tuple. Missing fields surface as empty
/// strings / None; the dispatcher discards calls whose ExtensionId is
/// empty.
pub fn ParsePayload(Payload:&Value) -> (String, String, String) {
	let ExtensionId = Payload.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Variable = Payload.get("variable").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Value_ = Payload.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	(ExtensionId, Variable, Value_)
}
