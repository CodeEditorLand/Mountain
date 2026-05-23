
//! Wire method: `search:findInFiles` / `search:textSearch`.
//! Delegates to `SearchProvider::TextSearch`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, mut Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Search::SearchProvider::SearchProvider;

	let QueryValue = if Arguments.first().map(|V| V.is_object()).unwrap_or(false) {
		Arguments.remove(0)
	} else if let Some(Pattern) = Arguments.first().and_then(|V| V.as_str()) {
		let IsRegex = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

		let IsCase = Arguments.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

		let IsWord = Arguments.get(3).and_then(|V| V.as_bool()).unwrap_or(false);

		json!({
			"pattern": Pattern,
			"isRegex": IsRegex,
			"isCaseSensitive": IsCase,
			"isWordMatch": IsWord,
		})
	} else {
		return Err("search:findInFiles requires pattern or TextSearchQuery".to_string());
	};

	let OptionsValue = Arguments.into_iter().next().unwrap_or(Value::Null);

	dev_log!("search", "search:textSearch delegating to SearchProvider::TextSearch");

	RunTime
		.Environment
		.TextSearch(QueryValue, OptionsValue)
		.await
		.map_err(|Error| Error.to_string())
}
