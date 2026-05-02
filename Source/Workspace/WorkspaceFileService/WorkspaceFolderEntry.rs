#![allow(non_snake_case)]

//! Single `folders` entry in a `.code-workspace` JSON document. Path is
//! relative to the workspace file's parent directory.

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(super) struct Struct {
	pub(super) path:String,
}
