//! Wire method: `search:findInFiles` / `search:textSearch`.
//! Delegates to `SearchProvider::TextSearch`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgBool,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, mut Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Search::SearchProvider::SearchProvider;

	let QueryValue = if Arguments.first().map(|V| V.is_object()).unwrap_or(false) {
		Arguments.remove(0)
	} else if let Some(Pattern) = Arguments.first().and_then(|V| V.as_str()) {
		let IsRegex = ArgBool(&Arguments, 1);

		let IsCase = ArgBool(&Arguments, 2);

		let IsWord = ArgBool(&Arguments, 3);

		json!({
			"pattern": Pattern,
			"isRegex": IsRegex,
			"isCaseSensitive": IsCase,
			"isWordMatch": IsWord,
		})
	} else {
		return Err("search:findInFiles requires pattern or TextSearchQuery".to_string());
	};

	let OptionsValue = Arguments.into_iter().Next().unwrap_or(Value::Null);

	dev_log!("search", "search:textSearch delegating to SearchProvider::TextSearch");

	RunTime
		.Environment
		.TextSearch(QueryValue, OptionsValue)
		.await
		.map_err(|Error| Error.to_string())
}
