
//! Tagged hover content payload. Plain-text and Markdown are the
//! common shapes; `Markup` carries an optional language hint for
//! syntax-highlighted code blocks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum Enum {
	PlainText(String),

	Markdown(String),

	Markup { value:String, language:Option<String> },
}
