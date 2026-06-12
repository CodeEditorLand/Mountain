//! Wire method: `keybinding:resolve`.
//!
//! Arguments: `[keyExpression, contextSnapshot]`. Resolves the single
//! best keybinding rule for a pressed key against a context-key snapshot:
//!
//! 1. Merge all rule sources via `KeybindingProvider::GetResolvedKeybinding`
//!    (extensions + dynamic registry + user `keybindings.json`).
//! 2. Keep rules whose normalised key expression matches the input (modifier
//!    aliases and ordering are canonicalised, chords compared
//!    stroke-for-stroke).
//! 3. Drop rules whose `when` clause evaluates false in the snapshot.
//! 4. Rank survivors by source weight (user > dynamic > extension), then
//!    when-clause specificity, so `editorTextFocus && !inQuickOpen` beats a
//!    bare `editorTextFocus` which beats an unguarded binding.
//!
//! Returns the winning rule (`{key, command, when?, args?, source?}`) or
//! `null` when nothing is bound for the key in this context.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::{Value, json};

use crate::{Environment::Utility::WhenClause, RunTime::ApplicationRunTime::ApplicationRunTime};

fn SourceWeight(Rule:&Value) -> u32 {
	match Rule.get("source").and_then(Value::as_str) {
		Some("user") => 3,

		Some(Source) if Source.starts_with("dynamic") => 2,

		Some(Source) if Source.starts_with("extension") => 1,

		_ => 0,
	}
}

fn WhenSpecificity(Rule:&Value) -> u32 {
	Rule.get("when")
		.and_then(Value::as_str)
		.and_then(|Clause| WhenClause::Parse(Clause).ok())
		.map(|Expression| WhenClause::SpecificityOf(&Expression))
		.unwrap_or(0)
}

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let KeyExpression = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:resolve requires a key expression".to_string())?;

	let EmptyContext = json!({});

	let Context = Arguments.get(1).filter(|V| V.is_object()).unwrap_or(&EmptyContext).clone();

	let NormalizedInput = WhenClause::NormalizeKeyExpression(KeyExpression);

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	let Rules = Provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())?;

	let Best = Rules
		.as_array()
		.map(|All| {
			All.iter()
				.filter(|Rule| {
					Rule.get("key")
						.and_then(Value::as_str)
						.is_some_and(|Key| WhenClause::NormalizeKeyExpression(Key) == NormalizedInput)
				})
				.filter(|Rule| WhenClause::EvaluateClause(Rule.get("when").and_then(Value::as_str), &Context))
				.max_by_key(|Rule| (SourceWeight(Rule), WhenSpecificity(Rule)))
				.cloned()
		})
		.unwrap_or(None);

	Ok(Best.unwrap_or(Value::Null))
}
