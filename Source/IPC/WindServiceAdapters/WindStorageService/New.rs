//! `WindStorageService::New`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};

pub fn Fn(provider:Arc<dyn StorageProvider>) -> Struct { Self { provider } }
