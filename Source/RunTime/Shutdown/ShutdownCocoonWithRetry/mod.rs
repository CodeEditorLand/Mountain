pub mod ShutdownCocoonWithRetry;

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Error::CommonError::CommonError, IPC::IPCProvider::IPCProvider};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

#[derive(Debug, Clone)]
pub struct Struct;
