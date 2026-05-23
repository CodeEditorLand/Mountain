//! # Documents Effect (CreateEffectForRequest)
//!
//! Effect constructors for the `Document.*` RPC family. Delegates to the
//! `DocumentProvider` trait on `MountainEnvironment` for save operations.
//!
//! ## Methods handled
//!
//! | Method | Description |
//! |---|---|
//! | `Document.Save` | Save the document at the given URI to disk |
//! | `Document.SaveAs` | Save the document to a new location specified by the caller |

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
						if uri_str.is_empty() {
							return Err("Document.Save: empty URI (resource not found)".to_string());
						}
						let uri = Url::parse(uri_str)
							.map_err(|e| format!("Document.Save: invalid URI '{}': {}", uri_str, e))?;
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
						if original_uri_str.is_empty() {
							return Err("Document.SaveAs: empty URI (resource not found)".to_string());
						}
						let original_uri = Url::parse(original_uri_str)
							.map_err(|e| format!("Document.SaveAs: invalid URI '{}': {}", original_uri_str, e))?;
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
