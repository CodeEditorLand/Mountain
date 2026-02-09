//! # EchoAction RPC Service
//!
//! EchoAction service for bidirectional actions and extension host routing.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// EchoAction server implementation
pub struct EchoActionServer {
    // Placeholder fields
}

impl EchoActionServer {
    pub fn new() -> Self {
        Self {}
    }
}

/// Extension host registry
pub struct ExtensionHostRegistry {
    hosts: Arc<RwLock<HashMap<String, String>>>,
}

impl ExtensionHostRegistry {
    pub fn new() -> Self {
        Self {
            hosts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for ExtensionHostRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension router
pub struct ExtensionRouter {
    registry: Arc<ExtensionHostRegistry>,
}

impl ExtensionRouter {
    pub fn new(registry: Arc<ExtensionHostRegistry>) -> Self {
        Self { registry }
    }
}
