#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$scm:createSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let resource = Parameters.get(0).cloned().unwrap_or_default();
						provider
							.CreateSourceControl(resource)
							.await
							.map(|handle| json!(handle))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:updateSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let update = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControl(handle, update)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:updateGroup" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let group_data = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControlGroup(handle, group_data)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:registerInputBox" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.RegisterInputBox(handle, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
