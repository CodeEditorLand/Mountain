pub mod Validate;
pub mod CreateMinimal;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ExtensionDescriptionStateDTO = Struct;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	pub Identifier:Value,
	pub Name:String,
	pub Version:String,
	pub Publisher:String,
	pub Engines:Value,
	pub Main:Option<String>,
	pub Browser:Option<String>,
	pub ModuleType:Option<String>,
	pub IsBuiltin:bool,
	pub IsUnderDevelopment:bool,
	pub ExtensionLocation:Value,
	pub ActivationEvents:Option<Vec<String>>,
	pub Contributes:Option<Value>,
	pub Categories:Option<Vec<String>>,
	pub DisplayName:Option<String>,
	pub Description:Option<String>,
	pub Keywords:Option<Vec<String>>,
	pub Repository:Option<Value>,
	pub Bugs:Option<Value>,
	pub Homepage:Option<String>,
	pub License:Option<String>,
	pub Icon:Option<String>,
	pub AiKey:Option<String>,
	pub ExtensionKind:Option<Value>,
	pub Capabilities:Option<Value>,
	pub ExtensionDependencies:Option<Vec<String>>,
	pub ExtensionPack:Option<Vec<String>>,
}
