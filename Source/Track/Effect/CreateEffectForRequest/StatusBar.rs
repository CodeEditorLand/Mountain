#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};

use serde_json::{Value, json};

use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {

		"$statusBar:set" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let text = Parameters.get(0).and_then(Value::as_str).unwrap_or("status").to_string();
						let entry = StatusBarEntryDTO {
							EntryIdentifier:"id".to_string(),
							ItemIdentifier:"item".to_string(),
							ExtensionIdentifier:"ext".to_string(),
							Name:None,
							Text:text,
							Tooltip:None,
							HasTooltipProvider:false,
							Command:None,
							Color:None,
							BackgroundColor:None,
							IsAlignedLeft:false,
							Priority:None,
							AccessibilityInformation:None,
						};
						provider
							.SetStatusBarEntry(entry)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$statusBar:dispose" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let id = Parameters.get(0).and_then(Value::as_str).unwrap_or("id").to_string();
						provider
							.DisposeStatusBarEntry(id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$setStatusBarMessage" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let message_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("msg_id").to_string();
						let text = Parameters.get(1).and_then(Value::as_str).unwrap_or("message").to_string();
						provider
							.SetStatusBarMessage(message_id, text)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$disposeStatusBarMessage" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let message_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("msg_id").to_string();
						provider
							.DisposeStatusBarMessage(message_id)
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
