pub mod New;

use std::sync::Arc;
use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Environment::Environment,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory},
	TreeView::TreeViewProvider::TreeViewProvider,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use url::Url;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[derive(Clone)]
pub struct Struct {
	AppicationHandle:AppHandle,

impl Environment for Struct {}
}
