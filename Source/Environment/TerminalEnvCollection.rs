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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MutatorType {
	Replace = 1,

	Append = 2,

	Prepend = 3,
}

#[derive(Clone, Debug)]
pub struct Mutator {
	pub Variable:String,

	pub Value:String,

	pub Kind:MutatorType,
}

#[derive(Clone, Default)]
pub struct ExtensionCollection {
	pub Persistent:bool,

	pub Description:Option<String>,

	pub Mutators:HashMap<String, Mutator>,
}

static REGISTRY:OnceLock<Mutex<HashMap<String, ExtensionCollection>>> = OnceLock::new();

fn Get() -> &'static Mutex<HashMap<String, ExtensionCollection>> { REGISTRY.get_or_init(|| Mutex::new(HashMap::new())) }

pub fn Replace(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Replace });
	}
}

pub fn Append(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Append });
	}
}

pub fn Prepend(ExtensionId:&str, Variable:String, Value:String) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry
			.Mutators
			.insert(Variable.clone(), Mutator { Variable, Value, Kind:MutatorType::Prepend });
	}
}

pub fn Delete(ExtensionId:&str, Variable:&str) {
	if let Ok(mut Guard) = Get().lock() {
		if let Some(Entry) = Guard.get_mut(ExtensionId) {
			Entry.Mutators.remove(Variable);
		}
	}
}

pub fn Clear(ExtensionId:&str) {
	if let Ok(mut Guard) = Get().lock() {
		if let Some(Entry) = Guard.get_mut(ExtensionId) {
			Entry.Mutators.clear();
		}
	}
}

pub fn SetPersistent(ExtensionId:&str, Persistent:bool) {
	if let Ok(mut Guard) = Get().lock() {
		let Entry = Guard.entry(ExtensionId.to_string()).or_default();

		Entry.Persistent = Persistent;
	}
}

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
