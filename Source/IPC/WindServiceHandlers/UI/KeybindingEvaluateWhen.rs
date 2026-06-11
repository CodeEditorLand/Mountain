//! Wire method: `keybinding:evaluateWhen`.
//!
//! Arguments: `[whenClause, contextSnapshot]`. Evaluates a context-key
//! when clause against the caller-supplied context object (Sky owns the
//! live context-key store; Mountain only sees snapshots). Returns a JSON
//! boolean. A missing/empty clause is `true`; an unparseable clause is
//! `false` (VS Code semantics for invalid expressions).

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{Environment::Utility::WhenClause, RunTime::ApplicationRunTime::ApplicationRunTime};

pub async fn Fn(_RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Clause = Arguments.first().and_then(|V| V.as_str());

	let EmptyContext = json!({});

	let Context = Arguments.get(1).filter(|V| V.is_object()).unwrap_or(&EmptyContext);

	Ok(json!(WhenClause::EvaluateClause(Clause, Context)))
}
