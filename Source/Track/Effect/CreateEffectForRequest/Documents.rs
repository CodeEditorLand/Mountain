#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Document::DocumentProvider::DocumentProvider, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Document.Save" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
						let uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						document_provider
							.SaveDocument(uri)
							.await
							.map(|success| json!(success))
							.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Document.SaveAs" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
						let original_uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let original_uri = Url::parse(original_uri_str)
							.unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						let target_uri = Parameters
							.get(1)
							.and_then(Value::as_str)
							.map(Url::parse)
							.transpose()
							.unwrap_or(None);
						document_provider
							.SaveDocumentAs(original_uri, target_uri)
							.await
							.map(|uri_option| json!(uri_option))
							.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
