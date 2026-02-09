//! # Configuration RPC Service
//!
//! Configuration management service for read/write operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration service
pub struct ConfigurationService {
    config: HashMap<String, serde_json::Value>,
}

impl ConfigurationService {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.config.insert(key, value);
    }
}

impl Default for ConfigurationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationScope {
    Global,
    Workspace,
    Folder,
}

/// Configuration update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdate {
    pub key: String,
    pub value: serde_json::Value,
    pub scope: ConfigurationScope,
}
