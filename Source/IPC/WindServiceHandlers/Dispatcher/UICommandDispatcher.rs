//! UI command dispatcher - handles decorations, commands, extensions, etc.

use serde_json::Value;

use crate::{
	Commands::{Execute::Fn as CommandsExecute, GetAll::Fn as CommandsGetAll},
	Extensions::{
		ExtensionsGet::Fn as ExtensionsGet,
		ExtensionsGetAll::Fn as ExtensionsGetAll,
		ExtensionsGetInstalled::Fn as ExtensionsGetInstalled,
		ExtensionsIsActive::Fn as ExtensionsIsActive,
	},
	UI::{
		DecorationsClear::Fn as DecorationsClear,
		DecorationsGet::Fn as DecorationsGet,
		DecorationsGetMany::Fn as DecorationsGetMany,
		DecorationsSet::Fn as DecorationsSet,
	},
	Utilities::JsonValueHelpers::{arg_string, arg_u64, json},
};

/// Dispatches UI commands.
pub async fn dispatch_ui_commands(
	app_handle:&tauri::AppHandle,

	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		// Commands
		"commands:execute" | "commands:executeCommand" => CommandsExecute(runtime.clone(), arguments).await,

		"commands:getAll" | "commands:getCommands" => CommandsGetAll(runtime.clone()).await,

		"commands:registerCommand"
		| "commands:unregisterCommand"
		| "commands:onDidRegisterCommand"
		| "commands:onDidExecuteCommand" => Ok(Value::Null),

		// Extensions
		"extensions:getAll" => ExtensionsGetAll(runtime.clone()).await,

		"extensions:get" => ExtensionsGet(runtime.clone(), arguments).await,

		"extensions:isActive" => ExtensionsIsActive(runtime.clone(), arguments).await,

		"extensions:activate" => {
			let extension_id = arg_string(&arguments, 0);

			if extension_id.is_empty() {
				Ok(Value::Null)
			} else {
				let notification =
					json!({ "event": format!("onCustom:{}", extension_id), "extensionId": extension_id });

				let _ = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$activateByEvent".to_string(),
					notification,
				)
				.await;

				Ok(Value::Null)
			}
		},

		"extensions:getInstalled" | "extensions:scanSystemExtensions" => {
			let effective_args = if command == "extensions:scanSystemExtensions" {
				let mut overridden = arguments.clone();

				if overridden.is_empty() {
					overridden.push(Value::Null);
				}

				overridden[0] = json!(0);

				overridden
			} else {
				arguments.clone()
			};

			ExtensionsGetInstalled(runtime.clone(), effective_args).await
		},

		"extensions:scanUserExtensions" => {
			let mut user_args = arguments.clone();

			if user_args.is_empty() {
				user_args.push(Value::Null);
			}

			user_args[0] = json!(1);

			ExtensionsGetInstalled(runtime.clone(), user_args).await
		},

		"extensions:getUninstalled" => Ok(Value::Array(Vec::new())),

		"extensions:query" | "extensions:getExtensions" | "extensions:getRecommendations" => {
			Ok(Value::Array(Vec::new()))
		},

		"extensions:getExtensionsControlManifest" => {
			Ok(json!({"malicious": [], "deprecated": {}, "search": [], "autoUpdate": {}}))
		},

		"extensions:resetPinnedStateForAllUserExtensions" => Ok(Value::Null),

		"extensions:install" => {
			crate::Extension::ExtensionInstall::Fn(app_handle.clone(), runtime.clone(), arguments).await
		},

		"extensions:uninstall" => {
			crate::Extension::ExtensionUninstall::Fn(app_handle.clone(), runtime.clone(), arguments).await
		},

		"extensions:getManifest" => Ok(Value::Null),

		"extensions:reinstall" | "extensions:updateMetadata" => Ok(Value::Null),

		// Decorations
		"decorations:get" => DecorationsGet(runtime.clone(), arguments).await,

		"decorations:getMany" => DecorationsGetMany(runtime.clone(), arguments).await,

		"decorations:set" => DecorationsSet(runtime.clone(), arguments).await,

		"decorations:clear" => DecorationsClear(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown UI command: {}", command)),
	}
}
