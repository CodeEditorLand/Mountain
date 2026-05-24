//! `ExtensionDescriptionStateDTO::CreateMinimal`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(Identifier:Value, Name:String, Version:String, Publisher:String) -> Result<Self, String> {
		let Description = Self {
			Identifier,

			Name:Name.clone(),

			Version:Version.clone(),

			Publisher:Publisher.clone(),

			Engines:serde_json::json!({ "vscode": "*" }),

			Main:None,

			Browser:None,

			ModuleType:None,

			IsBuiltin:false,

			IsUnderDevelopment:false,

			ExtensionLocation:serde_json::json!(null),

			ActivationEvents:None,

			Contributes:None,

			Categories:None,

			DisplayName:None,

			Description:None,

			Keywords:None,

			Repository:None,

			Bugs:None,

			Homepage:None,

			License:None,

			Icon:None,

			AiKey:None,

			ExtensionKind:None,

			Capabilities:None,

			ExtensionDependencies:None,

			ExtensionPack:None,
		};

		Description.Validate()?;

		Ok(Description)
	}
